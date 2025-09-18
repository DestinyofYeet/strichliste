use itertools::Itertools;
use sqlx::{query, query_as, Executor};

use crate::{
    backend::database::{DBError, DatabaseResponse, DatabaseType},
    models::{DatabaseId, UserDB},
};

use super::{GroupId, UserId};

#[derive(Debug)]
pub struct Group {
    #[allow(dead_code)]
    pub id: GroupId,
    pub members: Vec<UserDB>,
}

impl Group {
    pub async fn get<T>(conn: &mut T, gid: GroupId) -> DatabaseResponse<Self>
    where
        for<'a> &'a mut T: Executor<'a, Database = DatabaseType>,
    {
        let group_db = GroupDB::get(conn, gid).await?;

        let members = GroupDB::get_members(conn, group_db.id).await?;

        Ok(Self {
            id: group_db.id.into(),
            members,
        })
    }

    pub async fn get_user_group_id<T>(conn: &mut T, uid: UserId) -> DatabaseResponse<GroupId>
    where
        for<'a> &'a mut T: Executor<'a, Database = DatabaseType>,
    {
        Ok(GroupDB::get_single_group(conn, uid).await?.into())
    }

    #[allow(dead_code)]
    pub async fn get_groups<T>(conn: &mut T, uid: UserId) -> DatabaseResponse<Vec<Self>>
    where
        for<'a> &'a mut T: Executor<'a, Database = DatabaseType>,
    {
        let group_ids = GroupDB::get_groups(conn, uid).await?;

        let mut groups: Vec<Self> = Vec::new();

        for group_id in group_ids {
            groups.push(Group::get(conn, GroupId(group_id.id)).await?);
        }

        Ok(groups)
    }

    pub async fn get_group_id_for_multiple_users<T>(
        conn: &mut T,
        uids: &[UserId],
    ) -> DatabaseResponse<GroupId>
    where
        for<'a> &'a mut T: Executor<'a, Database = DatabaseType>,
    {
        Ok(
            match GroupDB::get_group_for_multiple_users_id(&mut *conn, uids).await? {
                Some(val) => val.id,
                None => {
                    GroupDB::create_group_for_multiple_users_id(&mut *conn, uids)
                        .await?
                        .id
                }
            }
            .into(),
        )
    }
}

pub struct GroupDB {
    pub id: DatabaseId,
}

impl From<DatabaseId> for GroupDB {
    fn from(id: DatabaseId) -> Self {
        GroupDB { id }
    }
}

impl GroupDB {
    pub async fn get_members<T>(conn: &mut T, gid: DatabaseId) -> DatabaseResponse<Vec<UserDB>>
    where
        for<'a> &'a mut T: Executor<'a, Database = DatabaseType>,
    {
        query_as!(
            UserDB,
            "
                select Users.*
                    from UserGroupMap
                join Users on Users.id = UserGroupMap.uid 
                    where UserGroupMap.gid = ?
            ",
            gid
        )
        .fetch_all(&mut *conn)
        .await
        .map_err(From::from)
    }
    pub async fn _create<T>(conn: &mut T, id: DatabaseId) -> DatabaseResponse<Self>
    where
        for<'a> &'a mut T: Executor<'a, Database = DatabaseType>,
    {
        query!(
            "
                insert or ignore into Groups
                (id)
                values
                (?)
            ",
            id
        )
        .execute(&mut *conn)
        .await
        .map_err(DBError::new)?;

        Ok(GroupDB { id })
    }
    pub async fn create<T>(conn: &mut T) -> DatabaseResponse<Self>
    where
        for<'a> &'a mut T: Executor<'a, Database = DatabaseType>,
    {
        query!(
            "
              insert into Groups
              default values
              returning id  
            "
        )
        .fetch_one(&mut *conn)
        .await
        .map_err(From::from)
        .map(|e| e.id)
        .map(From::from)
    }

    pub async fn link_user<T>(&self, conn: &mut T, user_id: UserId) -> DatabaseResponse<()>
    where
        for<'a> &'a mut T: Executor<'a, Database = DatabaseType>,
    {
        query!(
            "
                insert into UserGroupMap
                (gid, uid)
                values
                (?, ?)
                ",
            self.id,
            user_id.0
        )
        .execute(&mut *conn)
        .await
        .map_err(From::from)
        .map(|_| ())
    }
    /// get primitive group id
    pub async fn get_single_group<T>(conn: &mut T, user_id: UserId) -> DatabaseResponse<DatabaseId>
    where
        for<'a> &'a mut T: Executor<'a, Database = DatabaseType>,
    {
        let opt = query!(
            "
                select gid
                from UserGroupMap
                group by gid
                having count(uid) = 1 AND MAX(uid) = ?;
            ",
            user_id.0
        )
        .fetch_one(&mut *conn)
        .await
        .map_err(From::from)
        .map(|elem| elem.gid);
        match opt {
            Ok(Some(value)) => Ok(value),
            Ok(None) => Err(DBError::new(format!(
                "FATAL user with id {} did not have group",
                user_id
            ))),
            Err(e) => Err(e),
        }
    }
    pub async fn get_groups<T>(conn: &mut T, user_id: UserId) -> DatabaseResponse<Vec<Self>>
    where
        for<'a> &'a mut T: Executor<'a, Database = DatabaseType>,
    {
        query!(
            "
                select Groups.id as id
                from Groups
                join UserGroupMap on Groups.id = UserGroupMap.gid
                where UserGroupMap.uid = ?
            ",
            user_id.0
        )
        .fetch_all(&mut *conn)
        .await
        .map_err(From::from)
        .map(|elem| elem.into_iter().map(|elem| Self { id: elem.id }).collect())
    }

    pub async fn get<T>(conn: &mut T, gid: GroupId) -> DatabaseResponse<Self>
    where
        for<'a> &'a mut T: Executor<'a, Database = DatabaseType>,
    {
        let group = query_as!(
            Self,
            "
                select *
                from Groups
                where id = ?
            ",
            gid.0
        )
        .fetch_optional(&mut *conn)
        .await
        .map_err(DBError::new)?;

        match group {
            None => Err(DBError::new(format!("Failed to find group: {}", gid.0))),
            Some(value) => Ok(value),
        }
    }

    pub async fn get_group_for_multiple_users_id<T>(
        conn: &mut T,
        user_ids: &[UserId],
    ) -> DatabaseResponse<Option<Self>>
    where
        for<'a> &'a mut T: Executor<'a, Database = DatabaseType>,
    {
        let count = user_ids.len() as i64;
        let user_ids_string: String =
            Itertools::intersperse(user_ids.iter().map(ToString::to_string), String::from(", "))
                .collect();

        let group_id = query!(
            "
                select gid
                from UserGroupMap
                group by gid
                having
                    count(distinct uid) = ?
                    and sum(case when uid not in (?) then 1 else 0 end) = 0
                    and count(distinct case when uid in (?) then uid end) = ?
            ",
            count,
            user_ids_string,
            user_ids_string,
            count
        )
        .fetch_optional(&mut *conn)
        .await?;

        Ok(group_id.map(|val| GroupDB { id: val.gid }))
    }

    pub async fn create_group_for_multiple_users_id<T>(
        conn: &mut T,
        user_ids: &[UserId],
    ) -> DatabaseResponse<Self>
    where
        for<'a> &'a mut T: Executor<'a, Database = DatabaseType>,
    {
        let new_group = GroupDB::create(&mut *conn).await?;

        for user in user_ids.iter() {
            new_group.link_user(&mut *conn, *user).await?;
        }

        Ok(new_group)
    }
}

impl From<GroupDB> for GroupId {
    fn from(val: GroupDB) -> Self {
        GroupId(val.id)
    }
}

impl From<&GroupDB> for GroupId {
    fn from(value: &GroupDB) -> Self {
        Self(value.id)
    }
}
