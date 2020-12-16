use serde::Serialize;
use crate::database as db;
use deadpool_postgres::Pool;
use crate::utils::cache_short;

pub async fn get_group_info(group_id: db::GroupID, pool: Pool)
    -> Result<Box<dyn warp::Reply>, warp::Rejection>
{
    let group_info = match db::group_info(pool, group_id).await? {
        Some(info) => info,
        None => return Ok(Box::new(warp::http::StatusCode::NOT_FOUND))
    };
    Ok(Box::new(cache_short(warp::reply::json(&group_info))))
}

#[derive(Serialize)]
struct ChannelInfo {
    channel_id: db::ChannelID,
    name: String,
}

// Would it make sense for this endpoint to also return group info?
pub async fn get_group_channels(group_id: db::GroupID, pool: Pool)
    -> Result<impl warp::Reply, warp::Rejection>
{
    let rows = db::group_channels(pool, group_id).await?;
    Ok(cache_short(warp::reply::json(&rows
        .iter()
        .map(|row| ChannelInfo {
            channel_id: row.get(0),
            name: row.get(1),
        })
        .collect::<Vec<_>>()
    )))
}
