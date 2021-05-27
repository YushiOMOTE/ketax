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
#[serde(rename_all = "camelCase")]
pub struct Image {
    pub id: String,
    pub filename: Option<String>,
    pub tags: Option<Vec<String>>,
    pub access_count: Option<i32>,
    pub access_date: Option<String>,
    pub release_date: Option<String>,
}

#[derive(Clone, GraphQLInputObject, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct NewImage {
    pub id: String,
    pub filename: Option<String>,
    pub tags: Option<Vec<String>>,
    pub access_count: Option<i32>,
    pub access_date: Option<String>,
    pub release_date: Option<String>,
}

impl From<NewImage> for Image {
    fn from(new: NewImage) -> Self {
        Self {
            id: new.id,
            filename: new.filename,
            tags: new.tags,
            access_count: new.access_count,
            access_date: new.access_date,
            release_date: new.release_date,
        }
    }
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
    fn version() -> String {
        "1.0".into()
    }

    #[graphql(arguments(ids(description = "list of the ids of the images")))]
    fn images(ctx: &Context, ids: Option<Vec<String>>) -> FieldResult<Vec<Image>> {
        let imgs = match ids {
            Some(ids) => {
                let imgs: Result<Vec<_>, _> = ids
                    .into_iter()
                    .map(|id| ctx.db.get(&id))
                    .filter_map(|i| i.transpose())
                    .collect();
                imgs?
            }
            None => ctx.db.list()?,
        };
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
