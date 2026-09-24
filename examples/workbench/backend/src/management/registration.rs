use super::response::JSON_CONTENT_TYPE;
use gateway_plugin_sdk::call::management::{
    ManagementPage, ManagementRegistration, ManagementResource, ManagementRoute,
};

pub(crate) fn registration() -> ManagementRegistration {
    let get = |path: &str| ManagementRoute {
        method: "GET".to_owned(),
        path: path.to_owned(),
        request_content_types: Vec::new(),
        response_content_types: vec![JSON_CONTENT_TYPE.to_owned()],
    };
    let post = |path: &str| ManagementRoute {
        method: "POST".to_owned(),
        path: path.to_owned(),
        request_content_types: vec![JSON_CONTENT_TYPE.to_owned()],
        response_content_types: vec![JSON_CONTENT_TYPE.to_owned()],
    };
    ManagementRegistration {
        routes: vec![
            get("api/snapshot"),
            post("api/echo"),
            post("api/models"),
            get("api/tasks"),
            post("api/tasks"),
            post("api/fetch-text"),
            post("api/log"),
        ],
        resources: ["web/index.html", "web/app.js", "web/app.css"]
            .into_iter()
            .map(|path| ManagementResource {
                path: path.to_owned(),
                public: path == "web/app.css",
            })
            .collect(),
        pages: vec![ManagementPage {
            id: "capability-workbench".to_owned(),
            title: "插件工作台".to_owned(),
            description: Some("体验请求、账号与管理扩展".to_owned()),
            entry: "web/index.html".to_owned(),
            icon: None,
        }],
        callbacks: Vec::new(),
    }
}
