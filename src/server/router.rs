use std::{collections::HashMap, sync::Arc};

use thiserror::Error;

use crate::HandlerTypes;

#[derive(Clone)]
pub struct RouterService {
    pub router: Arc<Router>,
}

pub struct RouteMatch<'a> {
    pub handler: &'a HandlerTypes,
    pub params: HashMap<String, String>,
}
#[derive(Error, Debug)]
pub enum RouterError {
    #[error("failed to find appropriate route")]
    PathNotFound,
}
pub struct RouterBuilder {
    routes: Vec<(&'static str, HandlerTypes)>,
}

pub struct Router {
    inner: Vec<(&'static str, HandlerTypes)>,
}

impl RouterBuilder {
    pub fn new() -> Self {
        Self { routes: Vec::new() }
    }

    pub fn route(mut self, path: &'static str, handler: HandlerTypes) -> Self {
        self.routes.push((path, handler));
        self
    }

    pub fn build(self) -> Router {
        Router { inner: self.routes }
    }
}

impl Router {
    pub fn builder() -> RouterBuilder {
        RouterBuilder::new()
    }

    pub fn into_service(self) -> RouterService {
        RouterService {
            router: Arc::new(self),
        }
    }
}

impl Router {
    pub fn matches(&self, path: &str) -> Option<RouteMatch> {
        // let normalized = normalize_path(path);

        self.inner.iter().find_map(|(pattern, handler)| {
            let path_pattern = PatternPath::from_path(pattern);
            if path_pattern.matches(&path) {
                Some(RouteMatch {
                    handler,
                    params: path_pattern.extract_params(&path),
                })
            } else {
                None
            }
        })
    }
}

#[derive(Debug)]
pub struct PatternPath {
    segments: Vec<PathSegment>,
}

#[derive(Debug)]
enum PathSegment {
    Static(String),
    Parameter(String),
}

impl PatternPath {
    fn from_path(path: &str) -> Self {
        let segments = path
            .split("/")
            .filter(|s| !s.is_empty())
            .map(|segment| {
                if segment.starts_with("{") && segment.ends_with("}") {
                    PathSegment::Parameter(segment[1..segment.len() - 1].to_string())
                } else {
                    //normal path
                    PathSegment::Static(segment.to_string())
                }
            })
            .collect();
        PatternPath { segments }
    }

    fn matches(&self, path: &str) -> bool {
        let path_segments: Vec<_> = path.split("/").filter(|s| !s.is_empty()).collect();

        if path_segments.len() != self.segments.len() {
            return false;
        }

        self.segments
            .iter()
            .zip(path_segments)
            .all(|(pattern, segment)| match pattern {
                PathSegment::Static(s) => s == segment,
                PathSegment::Parameter(_) => true,
            })
    }

    pub fn extract_params(&self, path: &str) -> HashMap<String, String> {
        let mut params = HashMap::new();
        let path_segments: Vec<_> = path.split("/").filter(|s| !s.is_empty()).collect();

        for (pattern, path_seg) in self.segments.iter().zip(path_segments) {
            if let PathSegment::Parameter(name) = pattern {
                params.insert(name.to_string(), path_seg.to_string());
            }
        }
        params
    }
}
