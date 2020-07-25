// AWS lambda runtime
#[macro_use]
extern crate lambda_runtime as lambda;
// Serde JSON serializer
#[macro_use]
extern crate serde_derive;
// Logging
#[macro_use]
extern crate log;
extern crate simple_logger;

use std::error::Error;

// --------------------------------------------------------------------------------
// Load .env configuration at compile time
// --------------------------------------------------------------------------------
use load_dotenv::load_dotenv;
load_dotenv!();

// --------------------------------------------------------------------------------
// Github GraphQL Api
// --------------------------------------------------------------------------------
use graphql_client::*;
use reqwest::blocking::Client;

type URI = String;

#[derive(GraphQLQuery)]
#[graphql(
    schema_path = "rsrc/schema.github.graphql",
    query_path = "rsrc/github.repos.graphql",
    response_derives = "Debug"
)]
struct RepoView;

fn fetch_repos(number: u16) -> Result<repo_view::ResponseData, Box<dyn Error>> {
    let api_token = env!("GITHUB_API_TOKEN");
    let client = Client::builder().user_agent("Maxi").build()?;
    let query = RepoView::build_query(repo_view::Variables { top_repo: number as i64, top_lang: 7 });
    let request = client
        .post("https://api.github.com/graphql")
        .bearer_auth(api_token)
        .json(&query);
    let body: Response<repo_view::ResponseData> = request.send()?.json()?;
    return if body.errors.is_some() {
        Err("Error while fetching github API.".into())
    } else {
        Ok(body.data.expect("missing response data"))
    };
}

// --------------------------------------------------------------------------------
// Convert to appropriate response
// --------------------------------------------------------------------------------
use std::collections::HashMap;
use std::convert::identity;

#[derive(Serialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
struct Skill {
    name: String,
    code_size: u64,
    color: String,
}

static DEFAULT_COLOR: &str = "#000";

macro_rules! repos_to_skills {
    ($nodes:expr, $exclude:expr) => {
        $nodes
            .into_iter()
            .filter_map(identity)
            .filter(|r| !$exclude.contains(&r.name.to_owned()))
            .flat_map(|r| r.languages)
            .flat_map(|l| l.edges)
            .flatten()
            .filter_map(identity)
            .map(|lang| Skill{
                name: lang.node.name.to_owned(),
                code_size: lang.size as u64,
                color: lang.node.color.as_ref().unwrap_or(&DEFAULT_COLOR.to_string()).to_string(),
            })
    }
}

fn extract_skills(data: repo_view::ResponseData, exclude: HashSet<String>) -> Vec<Skill> {
    let personal = repos_to_skills!(data.viewer.repositories.nodes.unwrap_or(vec![]) , &exclude);
    let contributions = repos_to_skills!(data.viewer.repositories_contributed_to.nodes.unwrap_or(vec![]), &exclude);
    let combined = personal.chain(contributions);
    let skills: HashMap<String, Skill> = combined.fold(HashMap::new(), |mut map, skill| {
        map.entry(skill.name.to_owned())
           .and_modify(|s| s.code_size += skill.code_size)
           .or_insert(skill);
        map
    });
    skills.values().cloned().collect()
}

// --------------------------------------------------------------------------------
// Lambda Handler
// --------------------------------------------------------------------------------
use lambda::error::HandlerError;
use std::collections::HashSet;

#[derive(Deserialize, Clone)]
struct APIRequest {
    top: Option<u16>,
    exclude: Option<HashSet<String>>
}

#[derive(Serialize, Clone, Debug)]
struct APIResponse {
    skills: Vec<Skill>,
}

fn request_handler(e: APIRequest, c: lambda::Context) -> Result<APIResponse, HandlerError> {
    let top = e.top.unwrap_or(50);
    if top == 0 {
        error!("Requesting zero github skills in request {}", c.aws_request_id);
        return Err("No skills requested.".into());
    }
    let exclude = e.exclude.unwrap_or(HashSet::new());
    let repos = match fetch_repos(top) {
        Ok(value) => value,
        Err(err) => return Err(err.to_string().as_str().into()),
    };
    let skills = extract_skills(repos, exclude);
    Ok(APIResponse { skills })
}

// --------------------------------------------------------------------------------
// Main Entrypoint
// --------------------------------------------------------------------------------
fn main() -> Result<(), Box<dyn Error>> {
    simple_logger::init_with_level(log::Level::Warn)?;
    lambda!(request_handler);
    Ok(())
}

// --------------------------------------------------------------------------------
// Tests
// --------------------------------------------------------------------------------
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn handler_returns_ok_with_argument() {
        let req = APIRequest { top: Some(2), exclude: None };
        let result = request_handler(req, Default::default());
        assert!(result.is_ok());
    }

    #[test]
    fn handler_returns_ok_without_argument() {
        let req = APIRequest { top: None, exclude: None };
        let result = request_handler(req, Default::default());
        assert!(result.is_ok());
    }

    #[test]
    fn handler_returns_error_with_zero_argument() {
        let req = APIRequest { top: Some(0), exclude: None };
        let result = request_handler(req, Default::default());
        assert!(result.is_err());
    }
}
