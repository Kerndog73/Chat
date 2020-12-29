use serde::Serialize;
use crate::error::Error;
use deadpool_postgres::Pool;
use super::{Channel, UserID};

pub type GroupID = i32;

/// Create a new group.
///
/// Returns Ok(None) if the name is not unique.
/// Returns Err if a database error occurred.
pub async fn create_group(pool: Pool, name: String, picture: String)
    -> Result<Option<GroupID>, Error>
{
    let conn = pool.get().await?;
    let stmt = conn.prepare("
        INSERT INTO Groop (name, picture)
        SELECT $1, $2
        WHERE NOT EXISTS (
            SELECT *
            FROM Groop
            WHERE name = $1
        )
        RETURNING group_id
    ").await?;
    Ok(conn.query_opt(&stmt, &[&name, &picture]).await?.map(|row| row.get(0)))
}

/// Get the channels in a group
///
/// Returns an empty vector if the group is invalid.
pub async fn group_channels(pool: Pool, group_id: GroupID)
    -> Result<Vec<Channel>, Error>
{
    let conn = pool.get().await?;
    let stmt = conn.prepare("
        SELECT channel_id, name
        FROM Channel
        WHERE group_id = $1
        ORDER BY channel_id
    ").await?;
    Ok(conn.query(&stmt, &[&group_id])
        .await?
        .iter()
        .map(|row| Channel {
            channel_id: row.get(0),
            name: row.get(1),
        })
        .collect())
}

#[derive(Serialize)]
pub struct Group {
    pub group_id: GroupID,
    pub name: String,
    pub picture: String,
}

/// Get the list of groups that a user is a member of.
pub async fn group_list(pool: Pool, user_id: UserID) -> Result<Vec<Group>, Error> {
    let conn = pool.get().await?;
    let stmt = conn.prepare("
        SELECT Groop.group_id, name, COALESCE(picture, '')
        FROM Groop
        JOIN Membership ON Membership.group_id = Groop.group_id
        WHERE Membership.user_id = $1
        ORDER BY Groop.group_id
    ").await?;
    Ok(conn.query(&stmt, &[&user_id]).await?.iter().map(|row| Group {
        group_id: row.get(0),
        name: row.get(1),
        picture: row.get(2),
    }).collect())
}

/// Determine whether a user is a member of a group
pub async fn group_member(pool: Pool, user_id: UserID, group_id: GroupID)
    -> Result<bool, Error>
{
    let conn = pool.get().await?;
    let stmt = conn.prepare("
        SELECT 1
        FROM Membership
        WHERE user_id = $1
        AND group_id = $2
    ").await?;
    Ok(conn.query_opt(&stmt, &[&user_id, &group_id]).await?.is_some())
}
