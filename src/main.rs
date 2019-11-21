use failure::*;
use graphql_client::*;
use log::*;
use prettytable::*;
use serde::*;
use structopt::StructOpt;

type URI = String;

#[derive(GraphQLQuery)]
#[graphql(
    schema_path = "src/schema.public.graphql",
    query_path = "src/query_1.graphql",
    response_derives = "Debug"
)]
struct LastPullRequest;

#[derive(GraphQLQuery)]
#[graphql(
    schema_path = "src/schema.public.graphql",
    query_path = "src/query_1.graphql",
    response_derives = "Debug"
)]
struct AssignAuthor;

#[derive(GraphQLQuery)]
#[graphql(
    schema_path = "src/schema.public.graphql",
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

    let (pr_id, suggested_id, author_id) = {
        let q0 = LastPullRequest::build_query(last_pull_request::Variables {
            name: name.to_string(),
            owner: owner.to_string(),
        });

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

        let response_prs = response_body
            .data
            .expect("response data")
            .repository
            .expect("repository")
            .pull_requests
            .nodes
            .expect("nodes");
        assert!(response_prs.len() == 1);
        let pr = &response_prs.last().unwrap().as_ref().expect("some node");
        let pr_id = pr.id.clone();
        let suggested_id = pr.suggested_reviewers.first().map(|rev| {
            rev.as_ref()
                .expect("suggestion contains reviewer")
                .reviewer
                .id
                .clone()
        });
        use crate::last_pull_request::LastPullRequestRepositoryPullRequestsNodesAuthorOn::User;
        let author_id = pr.author.as_ref().and_then(|auth| {
            if let User(user) = &auth.on {
                Some(user.id.clone())
            } else {
                None
            }
        });

        println!("{:?}\t{:?}\t🌟", pr_id, suggested_id);
        (pr_id, suggested_id, author_id)
    };

    let assign = {
        let q = AssignAuthor::build_query(assign_author::Variables {
            input: assign_author::AssignAuthorInput {
                assignable_id: pr_id.clone(),
                assignee_ids: vec![author_id.clone()],
                client_mutation_id: None,
            },
        });

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

        let response = response_body.data.expect("response data");
        response
    };

    //    let q1 = RequestReviews::build_query(request_reviews::Variables {
    //        input: request_reviews::RequestReviewsInput {
    //            client_mutation_id: None,
    //            pull_request_id: pull_id,
    //            team_ids: Some(vec![]),
    //            union: None,
    //            user_ids: Some(vec!["1".to_string()]),
    //        },
    //    });

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
