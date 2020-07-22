# GitHub Resume Skill Lambda

Small AWS lambda written in rust for generating resume skill 'tags'
from the languages used in your github repositories.

## Usage

1. Generate a personal github access token in the [Settings View](https://github.com/settings/tokens/new).
2. Create a `.env` file in the project root with content `GITHUB_API_TOKEN=[token]`
3. Run `make all` to compile the project for aws lambda using docker. The docker daemon has to be running.
4. Upload the resulting zip file `target/lambda-rust.zip` to your lambda.

## Links & References

- [AWS Lambda + Rust Setup guide used](https://aws.amazon.com/blogs/opensource/rust-runtime-for-aws-lambda)
