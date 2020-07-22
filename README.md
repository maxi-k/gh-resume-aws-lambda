# GitHub Resume Skill Lambda

Small AWS lambda written in rust for generating resume skill 'tags'
from the languages used in your github repositories.

TBD: Generate an image for the new github readmes.

## Usage

1. Generate a personal github access token in the [Settings View](https://github.com/settings/tokens/new).
2. Create a `.env` file in the project root with content `GITHUB_API_TOKEN=[token]`
3. Run `make all` to compile the project for aws lambda using docker. The docker daemon has to be running.
4. Upload the resulting zip file `target/lambda-rust.zip` to your lambda.

## Example Response

An example JSON response looks like this:

```json
{
  "skills": [
    {
      "name": "Assembly",
      "code_size": 87854,
      "color": "#6E4C13"
    },
    {
      "name": "Java",
      "code_size": 5946,
      "color": "#b07219"
    },
    {
      "name": "Makefile",
      "code_size": 29767,
      "color": "#427819"
    },
    {
      "name": "Dockerfile",
      "code_size": 716,
      "color": "#384d54"
    },
    {
      "name": "C++",
      "code_size": 7787412,
      "color": "#f34b7d"
    },
    {
      "name": "HTML",
      "code_size": 3312245,
      "color": "#e34c26"
    },
    {
      "name": "CMake",
      "code_size": 32868,
      "color": "#000"
    },
    {
      "name": "Emacs Lisp",
      "code_size": 372,
      "color": "#c065db"
    },
    {
      "name": "Rust",
      "code_size": 17163,
      "color": "#dea584"
    },
    {
      "name": "JavaScript",
      "code_size": 12741,
      "color": "#f1e05a"
    },
    {
      "name": "Go",
      "code_size": 27237,
      "color": "#00ADD8"
    }
  ]
}
```

## Links & References

- [AWS Lambda + Rust Setup guide used](https://aws.amazon.com/blogs/opensource/rust-runtime-for-aws-lambda)
