#!/usr/bin/env bash
set -euo pipefail

#Get {
#    base: String,
#    #[arg(short, long)]
#    path: Option<String>,
#    #[arg(short, long)]
#    query: Vec<String>,
#    #[arg(long)]
#    headers: Vec<String>,
#},

#Post {
#    json: Vec<String>,
#},

# TEST=$(cat << EOF
# group auth
#     test abc
#         get "/foo"
#         assert status 200
#         assert body.foo
#     end
# end

# group foo
#     test ab
#         get "/foo"
#         assert status 200
#         assert body.foo "bar"
#     end
#     test abc
#         post "http://localhost:8080"
#             --path "foo"
#             --headers "Content-type: application/json"

#         assert status 200
#     end
# end
# EOF
# )

# cargo run -- test "$TEST"

# vim: ft=bash
