use sqlx::SqlitePool;

use crate::db::models::{RoleRow, UserRoleRow};

/// Parameters for creating a new role (avoids too-many-arguments warning).
pub struct CreateRoleParams<'a> {
    pub id: &'a str,
    pub server_id: &'a str,
    pub name: &'a str,
    pub color: Option<&'a str>,
    pub icon_url: Option<&'a str>,
    pub position: i32,
    pub permissions: i64,
    pub is_default: bool,
}

/// Create a new role in a server.
pub async fn create_role(
    pool: &SqlitePool,
    params: &CreateRoleParams<'_>,
) -> Result<(), sqlx::Error> {
    sqlx::query(
        "INSERT INTO roles (id, server_id, name, color, icon_url, position, permissions, is_default) \
         VALUES (?, ?, ?, ?, ?, ?, ?, ?)",
    )
    .bind(params.id)
    .bind(params.server_id)
    .bind(params.name)
    .bind(params.color)
    .bind(params.icon_url)
    .bind(params.position)
    .bind(params.permissions)
    .bind(params.is_default as i32)
    .execute(pool)
    .await?;
    Ok(())
}

/// Get a role by ID.
pub async fn get_role(
    pool: &SqlitePool,
    role_id: &str,
) -> Result<Option<RoleRow>, sqlx::Error> {
    sqlx::query_as::<_, RoleRow>("SELECT * FROM roles WHERE id = ?")
        .bind(role_id)
        .fetch_optional(pool)
        .await
}

/// List all roles in a server, ordered by position descending.
pub async fn list_roles(
    pool: &SqlitePool,
    server_id: &str,
) -> Result<Vec<RoleRow>, sqlx::Error> {
    sqlx::query_as::<_, RoleRow>(
        "SELECT * FROM roles WHERE server_id = ? ORDER BY position DESC",
    )
    .bind(server_id)
    .fetch_all(pool)
    .await
}

/// Get the default (@everyone) role for a server.
pub async fn get_default_role(
    pool: &SqlitePool,
    server_id: &str,
) -> Result<Option<RoleRow>, sqlx::Error> {
    sqlx::query_as::<_, RoleRow>(
        "SELECT * FROM roles WHERE server_id = ? AND is_default = 1",
    )
    .bind(server_id)
    .fetch_all(pool)
    .await
    .map(|mut rows| rows.pop())
}

/// Update a role's properties.
pub async fn update_role(
    pool: &SqlitePool,
    role_id: &str,
    name: &str,
    color: Option<&str>,
    icon_url: Option<&str>,
    permissions: i64,
) -> Result<(), sqlx::Error> {
    sqlx::query(
        "UPDATE roles SET name = ?, color = ?, icon_url = ?, permissions = ? WHERE id = ?",
    )
    .bind(name)
    .bind(color)
    .bind(icon_url)
    .bind(permissions)
    .bind(role_id)
    .execute(pool)
    .await?;
    Ok(())
}

/// Update a role's position.
pub async fn update_role_position(
    pool: &SqlitePool,
    role_id: &str,
    position: i32,
) -> Result<(), sqlx::Error> {
    sqlx::query("UPDATE roles SET position = ? WHERE id = ?")
        .bind(position)
        .bind(role_id)
        .execute(pool)
        .await?;
    Ok(())
}

/// Delete a role by ID.
pub async fn delete_role(pool: &SqlitePool, role_id: &str) -> Result<(), sqlx::Error> {
    sqlx::query("DELETE FROM roles WHERE id = ?")
        .bind(role_id)
        .execute(pool)
        .await?;
    Ok(())
}

/// Assign a role to a user in a server.
pub async fn assign_role(
    pool: &SqlitePool,
    server_id: &str,
    user_id: &str,
    role_id: &str,
) -> Result<(), sqlx::Error> {
    sqlx::query(
        "INSERT OR IGNORE INTO user_roles (server_id, user_id, role_id) VALUES (?, ?, ?)",
    )
    .bind(server_id)
    .bind(user_id)
    .bind(role_id)
    .execute(pool)
    .await?;
    Ok(())
}

/// Remove a role from a user in a server.
pub async fn remove_role(
    pool: &SqlitePool,
    server_id: &str,
    user_id: &str,
    role_id: &str,
) -> Result<(), sqlx::Error> {
    sqlx::query(
        "DELETE FROM user_roles WHERE server_id = ? AND user_id = ? AND role_id = ?",
    )
    .bind(server_id)
    .bind(user_id)
    .bind(role_id)
    .execute(pool)
    .await?;
    Ok(())
}

/// Get all roles assigned to a user in a server.
pub async fn get_user_roles(
    pool: &SqlitePool,
    server_id: &str,
    user_id: &str,
) -> Result<Vec<RoleRow>, sqlx::Error> {
    sqlx::query_as::<_, RoleRow>(
        "SELECT r.* FROM roles r \
         JOIN user_roles ur ON r.id = ur.role_id \
         WHERE ur.server_id = ? AND ur.user_id = ? \
         ORDER BY r.position DESC",
    )
    .bind(server_id)
    .bind(user_id)
    .fetch_all(pool)
    .await
}

/// Get all user-role assignments for a server (for bulk loading).
pub async fn get_all_user_roles(
    pool: &SqlitePool,
    server_id: &str,
) -> Result<Vec<UserRoleRow>, sqlx::Error> {
    sqlx::query_as::<_, UserRoleRow>(
        "SELECT * FROM user_roles WHERE server_id = ?",
    )
    .bind(server_id)
    .fetch_all(pool)
    .await
}

/// Check if a server has any roles defined.
pub async fn server_has_roles(
    pool: &SqlitePool,
    server_id: &str,
) -> Result<bool, sqlx::Error> {
    let count: i64 =
        sqlx::query_scalar("SELECT COUNT(*) FROM roles WHERE server_id = ?")
            .bind(server_id)
            .fetch_one(pool)
            .await?;
    Ok(count > 0)
}
