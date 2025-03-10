use std::sync::Arc;

//use actix_session::Session;
use actix_web::{web, HttpRequest, HttpResponse};
use config::Config;
use localization::TranslationDict;

use crate::{
    templates, user_settings, {BaseData, Site},
};

/// News page
pub async fn news(
    locale_dict: web::Data<Arc<TranslationDict>>,
    config: web::Data<Config>,
    request: HttpRequest,
) -> Result<HttpResponse, actix_web::Error> {
    let settings = user_settings::parse(&request);

    //session::init(&session, &settings);

    let news = resources::news::get()
        .last_entries(5)
        .cloned()
        .collect::<Vec<_>>();

    Ok(HttpResponse::Ok().body(
        render!(
            templates::base,
            BaseData::new(&locale_dict, settings, &config.asset_hash, &config)
                .with_site(Site::News(news))
        )
        .render(),
    ))
}
