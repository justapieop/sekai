use bigdecimal::{BigDecimal, FromPrimitive};
use moka::future::Cache;
use serde::Serialize;
use sqlx::{FromRow, Postgres, Transaction};
use std::error::Error;
use std::time::Duration;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, FromRow)]
pub struct DBComment {
    id: BigDecimal,
    user_id: Uuid,
    post_id: BigDecimal,
    reply_to: Option<BigDecimal>,
    content: String,
    attachment_id: Option<BigDecimal>,
}
pub struct CommentRepo {
    cache: Cache<u128, Vec<DBComment>>,
}

impl CommentRepo {
    pub fn new() -> Self {
        Self {
            cache: Cache::builder()
                .max_capacity(1000)
                .time_to_live(Duration::from_hours(24))
                .build(),
        }
    }

    pub async fn get_post_comments(
        &self,
        pool: &mut Transaction<'_, Postgres>,
        post_id: u128,
    ) -> Option<Vec<DBComment>> {
        if let Some(cached_comment_list) = self.cache.get(&post_id).await {
            return Some(cached_comment_list);
        }

        let comment_list: &Vec<DBComment> = &(match sqlx::query_as!(
            DBComment,
            r#"SELECT * FROM post_comments WHERE post_id = $1;"#,
            BigDecimal::from_u128(post_id).unwrap_or_default(),
        )
        .fetch_all(&mut **pool)
        .await
        {
            Ok(s) => s,
            Err(_) => return None,
        });

        self.cache.insert(post_id, comment_list.clone()).await;

        Some(comment_list.clone())
    }

    pub async fn post_comment(
        &self,
        pool: &mut Transaction<'_, Postgres>,
        id: u128,
        post_id: u128,
        user_id: Uuid,
        content: &str,
        attachment_id: Option<u128>,
        reply_to: Option<u128>,
    ) -> Result<DBComment, Box<dyn Error>> {
        let comment: &DBComment = &(match sqlx::query_as!(
            DBComment,
            r#"INSERT INTO post_comments (id, user_id, post_id, content, attachment_id, reply_to) VALUES ($1, $2, $3, $4, $5, $6) RETURNING *;"#,
            BigDecimal::from_u128(id).unwrap_or_default(),
            user_id,
            BigDecimal::from_u128(post_id).unwrap_or_default(),
            content,
            attachment_id.map(BigDecimal::from),
            reply_to.map(BigDecimal::from)
        ).fetch_one(&mut **pool).await {
            Ok(s) => s,
            Err(e) => return Err(e.into())
        });

        if let Some(mut cached_comment_list) = self.cache.get(&post_id).await {
            cached_comment_list.push(comment.clone());
            self.cache.insert(post_id, cached_comment_list).await;
        } else {
            let mut comment_list: Vec<DBComment> = Vec::new();
            comment_list.push(comment.clone());
            self.cache.insert(post_id, comment_list).await;
        }

        Ok(comment.clone())
    }
}
