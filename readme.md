
# Ideas

* [x] **0.1.0** `rgf <repo> --list`
  list all forks: List all forks of a repository.

* [x] **0.1.1** `rgf <repo> --add`
  add a fork: Add a fork to a repository.

* [x] **0.1.2**: View rate limits: Show the rate limits of the GitHub API.
  Not a local-fork manipulation usage but for general information.

* [ ] Conceptual: Be in a repository / Not in a repository?
  (see: `git rev-parse --is-inside-git-dir`)

  * [ ] If in a repository, use repository of 'origin' or the first entry of
    `git remote`.

  * [ ] If not in a repository, clone the repository and afterwards add all
    remotes to this new repository.
