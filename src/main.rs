use failure::*;
use graphql_client::*;
use log::*;
use prettytable::*;
use serde::*;
use structopt::StructOpt;

type URI = String;

#[derive(GraphQLQuery)]
#[graphql(
    schema_path = "src/schema.graphql",
    query_path = "src/query_1.graphql",
    response_derives = "Debug"
)]
struct LastPullRequest;

#[derive(GraphQLQuery)]
#[graphql(
    schema_path = "src/schema.graphql",
    query_path = "src/query_1.graphql",
    response_derives = "Debug"
)]
struct RequestReviews;

#[derive(StructOpt)]
#[structopt(author, about)]
struct Command {
    #[structopt(name = "repository")]
    repo: String,
    repo_token: String,
}

#[derive(Deserialize, Debug)]
struct Env {
    github_api_token: String,
}

fn parse_repo_name(repo_name: &str) -> Result<(&str, &str), failure::Error> {
    let mut parts = repo_name.split('/');
    match (parts.next(), parts.next()) {
        (Some(owner), Some(name)) => Ok((owner, name)),
        _ => Err(format_err!("wrong format for the repository name param (we expect something like facebook/graphql)"))
    }
}

fn main() -> Result<(), failure::Error> {
    dotenv::dotenv().ok();
    env_logger::init();

    //    let config: Env = envy::from_env().context("while reading from environment")?;

    let args = Command::from_args();

    let repo = args.repo;
    let (owner, name) = parse_repo_name(&repo).unwrap_or(("tomhoule", "graphql-client"));

    let repo_token = args.repo_token;

    let q0 = LastPullRequest::build_query(last_pull_request::Variables {
        name: name.to_string(),
        owner: owner.to_string(),
    });

//    let q1 = RequestReviews::build_query(request_reviews::Variables {
//        input: request_reviews::RequestReviewsInput {
//            client_mutation_id: None,
//            pull_request_id: pull_id,
//            team_ids: Some(vec![]),
//            union: None,
//            user_ids: Some(vec!["1".to_string()]),
//        },
//    });

    let client = reqwest::Client::new();

    let mut res = client
        .post("https://api.github.com/graphql")
        .bearer_auth(repo_token)
        .json(&q0)
        .send()?;

    let response_body: Response<last_pull_request::ResponseData> = res.json()?;
    info!("{:?}", response_body);

    if let Some(errors) = response_body.errors {
        println!("there are errors:");

        for error in &errors {
            println!("{:?}", error);
        }
    }

    let response_nodes = response_body.data.expect("response data").repository.expect("repository").pull_requests.nodes.expect("nodes");
    assert!(response_nodes.len() == 1);
    let node_id = &response_nodes.last().unwrap().as_ref().expect("some node").id;

    println!(
        "{:?}\t🌟",
        node_id
    );

//    let response_data = response_body
//        .data
//        .expect("missing response data")
//        .request_reviews
//        .expect("request_reviews");

//    println!(
//        "{:?}\t{:?}\t🌟",
//        response_data.client_mutation_id, response_data.pull_request
//    );

    /*
    let mut table = prettytable::Table::new();

    table.add_row(row!(b => "issue", "comments"));

    for issue in &response_data
        .repository
        .expect("missing repository")
        .issues
        .nodes
        .expect("issue nodes is null")
    {
        if let Some(issue) = issue {
            table.add_row(row!(issue.title, issue.comments.total_count));
        }
    }

    table.printstd();
    */
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_repo_name_works() {
        assert_eq!(
            parse_repo_name("graphql-rust/graphql-client").unwrap(),
            ("graphql-rust", "graphql-client")
        );
        assert!(parse_repo_name("abcd").is_err());
    }
}
