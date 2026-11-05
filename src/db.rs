use std::net::ToSocketAddrs;

use anyhow::{Context, Result, anyhow};
use serde_json::{Value, json};
#[cfg(windows)]
use tiberius::AuthMethod;
use tiberius::{Client, ColumnData, ColumnType, Config, Row, ToSql};
use tokio::net::TcpStream;
use tokio_util::compat::TokioAsyncWriteCompatExt;

pub(crate) async fn query_instance(
    instance: &str,
    database: Option<&str>,
    query: &str,
    params: &[&dyn ToSql],
) -> Result<String> {
    let (host, port) = match instance.rsplit_once(':') {
        Some((host, port)) if port.parse::<u16>().is_ok() => {
            (host, port.parse::<u16>().expect("validated above"))
        }
        _ => (instance, 1433),
    };
    let mut config = Config::new();
    config.host(host);
    config.port(port);
    if let Some(database) = database {
        config.database(database);
    }
    #[cfg(windows)]
    config.authentication(AuthMethod::Integrated);
    #[cfg(not(windows))]
    return Err(anyhow!(
        "Windows authentication is only available when running on Windows"
    ));
    config.trust_cert();
    let address = config.get_addr();
    let socket_addr = address
        .to_socket_addrs()
        .with_context(|| format!("Could not resolve SQL Server instance {instance}"))?
        .next()
        .ok_or_else(|| anyhow!("Could not resolve SQL Server instance {instance}"))?;
    let tcp = TcpStream::connect(socket_addr)
        .await
        .with_context(|| format!("Could not connect to SQL Server instance {instance}"))?;
    tcp.set_nodelay(true)?;
    let mut client = Client::connect(config, tcp.compat_write())
        .await
        .with_context(|| format!("SQL Server login failed for instance {instance}"))?;

    let rows = client
        .query(query, params)
        .await
        .context("SQL Server query failed")?
        .into_first_result()
        .await
        .context("Could not read SQL Server query results")?;

    let results: Vec<Value> = rows
        .iter()
        .map(|row| {
            let mut result = serde_json::Map::new();
            for (index, (column, value)) in row.cells().enumerate() {
                result.insert(
                    column.name().to_owned(),
                    column_data_to_json(row, index, column.column_type(), value),
                );
            }
            Value::Object(result)
        })
        .collect();
    Ok(serde_json::to_string(&results)?)
}

fn column_data_to_json(
    row: &Row,
    index: usize,
    column_type: ColumnType,
    value: &ColumnData<'_>,
) -> Value {
    match column_type {
        ColumnType::Datetime
        | ColumnType::Datetime4
        | ColumnType::Datetimen
        | ColumnType::Datetime2 => {
            return json!(
                row.try_get::<chrono::NaiveDateTime, _>(index)
                    .ok()
                    .flatten()
                    .map(|value| value.format("%Y-%m-%d %H:%M").to_string())
            );
        }
        ColumnType::DatetimeOffsetn => {
            return json!(
                row.try_get::<chrono::DateTime<chrono::FixedOffset>, _>(index)
                    .ok()
                    .flatten()
                    .map(|value| value.format("%Y-%m-%d %H:%M").to_string())
            );
        }
        _ => {}
    }

    match value {
        ColumnData::U8(value) => json!(value),
        ColumnData::I16(value) => json!(value),
        ColumnData::I32(value) => json!(value),
        ColumnData::I64(value) => json!(value),
        ColumnData::F32(value) => json!(value),
        ColumnData::F64(value) => json!(value),
        ColumnData::Bit(value) => json!(value),
        ColumnData::String(value) => json!(value.as_deref()),
        ColumnData::Guid(value) => json!(value.map(|value| value.to_string())),
        ColumnData::Binary(value) => json!(value.as_ref().map(|bytes| {
            bytes
                .iter()
                .map(|byte| format!("{byte:02x}"))
                .collect::<String>()
        })),
        ColumnData::Numeric(value) => json!(value.as_ref().map(|value| format!("{value:?}"))),
        ColumnData::Xml(value) => json!(value.as_ref().map(|value| format!("{value:?}"))),
        ColumnData::DateTime(value) => json!(value.map(|value| format!("{value:?}"))),
        ColumnData::SmallDateTime(value) => json!(value.map(|value| format!("{value:?}"))),
        ColumnData::Time(value) => json!(value.map(|value| format!("{value:?}"))),
        ColumnData::Date(value) => json!(value.map(|value| format!("{value:?}"))),
        ColumnData::DateTime2(value) => json!(value.map(|value| format!("{value:?}"))),
        ColumnData::DateTimeOffset(value) => json!(value.map(|value| format!("{value:?}"))),
    }
}
