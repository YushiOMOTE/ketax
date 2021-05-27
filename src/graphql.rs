use juniper::{
    graphql_object, EmptySubscription, FieldError, GraphQLInputObject, GraphQLObject, RootNode,
};
use serde::{Deserialize, Serialize};

use crate::db::Db;

type FieldResult<T> = std::result::Result<T, FieldError>;

fn timestamp() -> String {
    chrono::Local::now().naive_utc().format("%+").to_string()
}

#[derive(Clone, GraphQLObject, Deserialize, Serialize)]
pub struct Image {
    pub id: String,
    pub tags: Vec<String>,
    pub access_count: Option<i32>,
    pub access_date: Option<String>,
    pub release_date: Option<String>,
}

impl From<NewImage> for Image {
    fn from(new: NewImage) -> Self {
        Self {
            id: new.id,
            tags: new.tags.unwrap_or(vec![]),
            access_count: new.access_count,
            access_date: new.access_date,
            release_date: new.release_date,
        }
    }
}

#[derive(Clone, GraphQLInputObject, Deserialize, Serialize)]
pub struct NewImage {
    pub id: String,
    pub tags: Option<Vec<String>>,
    pub access_count: Option<i32>,
    pub access_date: Option<String>,
    pub release_date: Option<String>,
}

pub struct Context {
    db: Db,
}

impl Context {
    pub fn new(db: Db) -> Self {
        Self { db }
    }
}

impl juniper::Context for Context {}

pub struct Query;

#[graphql_object(context = Context)]
impl Query {
    fn apiVersion() -> String {
        "1.0".into()
    }

    #[graphql(arguments(limit(description = "limit")))]
    fn images(ctx: &Context, limit: Option<i32>) -> FieldResult<Vec<Image>> {
        let imgs = ctx.db.list(limit.map(|v| v as usize))?;
        Ok(imgs)
    }

    #[graphql(arguments(id(description = "id of the image")))]
    fn image(ctx: &Context, id: String) -> FieldResult<Option<Image>> {
        let img = ctx.db.get(&id)?;
        Ok(img)
    }
}

pub struct Mutation;

#[graphql_object(context = Context)]
impl Mutation {
    fn apiVersion() -> String {
        "1.0".into()
    }

    #[graphql(arguments(image(description = "set new image object")))]
    fn image(ctx: &Context, image: NewImage) -> FieldResult<Image> {
        ctx.db.set(&image.id, &image)?;
        Ok(image.into())
    }

    #[graphql(arguments(id(description = "id of the image")))]
    fn access(ctx: &Context, id: String) -> FieldResult<Option<Image>> {
        let img: Option<Image> = ctx.db.get(&id)?;
        match img {
            Some(mut img) => {
                img.access_count = Some(img.access_count.unwrap_or(0) + 1);
                img.access_date = Some(timestamp());
                ctx.db.set(&img.id, &img)?;
                Ok(Some(img))
            }
            None => Ok(None),
        }
    }
}

pub type Schema = RootNode<'static, Query, Mutation, EmptySubscription<Context>>;

pub fn schema() -> Schema {
    Schema::new(Query, Mutation, EmptySubscription::<Context>::new())
}
