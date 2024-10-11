# Contribution Guidelines

Note: The latest and most up-to-date documentation can be found on our [docs portal](https://docs.prylabs.network/docs/contribute/contribution-guidelines).

Excited by our work and want to get involved in building out our sharding releases? Or maybe you haven't learned as much about the Ethereum protocol but are a savvy developer? 

You can explore our [Open Issues](https://github.com/prysmaticlabs/prysm/issues) in-the works for our different releases. Feel free to fork our repo and start creating PR’s after assigning yourself to an issue of interest. We are always chatting on [Discord](https://discord.gg/CTYGPUJ) drop us a line there if you want to get more involved or have any questions on our implementation!

## Contribution Steps

**1. Set up Prysm following the instructions in README.md.**

**2. Fork the Prysm repo.**

Sign in to your GitHub account or create a new account if you do not have one already. Then navigate your browser to https://github.com/prysmaticlabs/prysm/. In the upper right hand corner of the page, click “fork”. This will create a copy of the Prysm repo in your account.

**3. Create a local clone of Prysm.**

```
$ mkdir -p $GOPATH/src/github.com/prysmaticlabs
$ cd $GOPATH/src/github.com/prysmaticlabs
$ git clone https://github.com/prysmaticlabs/prysm.git
$ cd $GOPATH/src/github.com/prysmaticlabs/prysm
```

**4. Link your local clone to the fork on your GitHub repo.**

```
$ git remote add myprysmrepo https://github.com/<your_github_user_name>/prysm.git
```

**5. Link your local clone to the Prysmatic Labs repo so that you can easily fetch future changes to the Prysmatic Labs repo.**

```
$ git remote add prysm https://github.com/prysmaticlabs/prysm.git
$ git remote -v (you should see myrepo and prysm in the list of remotes)
```

**6. Find an issue to work on.**

Check out open issues at https://github.com/prysmaticlabs/prysm/issues and pick one. Leave a comment to let the development team know that you would like to work on it. Or examine the code for areas that can be improved and leave a comment to the development team to ask if they would like you to work on it.

**7. Create a local branch with a name that clearly identifies what you will be working on.**

```
$ git checkout -b feature-in-progress-branch
```

**8. Make improvements to the code.**

Each time you work on the code be sure that you are working on the branch that you have created as opposed to your local copy of the Prysmatic Labs repo. Keeping your changes segregated in this branch will make it easier to merge your changes into the repo later.

```
$ git checkout feature-in-progress-branch
```

**9. Test your changes.**

Changes that only affect a single file can be tested with

```
$ go test <file_you_are_working_on>
```

**10. Stage the file or files that you want to commit.**

```
$ git add --all
```

This command stages all the files that you have changed. You can add individual files by specifying the file name or names and eliminating the “-- all”.

**11. Commit the file or files.**

```
$ git commit  -m “Message to explain what the commit covers”
```

You can use the –amend flag to include previous commits that have not yet been pushed to an upstream repo to the current commit.

**12. Fetch any changes that have occurred in the Prysmatic Labs Prysm repo since you started work.**

```
$ git fetch prysm
```

**13. Pull latest version of Prysm.**

```
$ git pull origin master
```

If there are conflicts between your edits and those made by others since you started work Git will ask you to resolve them. To find out which files have conflicts run ...

```
$ git status
```

Open those files one at a time, and you will see lines inserted by Git that identify the conflicts:

```
<<<<<< HEAD
Other developers’ version of the conflicting code
======
Your version of the conflicting code
'>>>>> Your Commit
```

The code from the Prysm repo is inserted between <<< and === while the change you have made is inserted between === and >>>>. Remove everything between <<<< and >>> and replace it with code that resolves the conflict. Repeat the process for all files listed by git status that have conflicts.

**14. Push your changes to your fork of the Prysm repo.**

Use git push to move your changes to your fork of the repo.

```
$ git push myrepo feature-in-progress-branch
```

**15. Check to be sure your fork of the Prysm repo contains your feature branch with the latest edits.**

Navigate to your fork of the repo on GitHub. On the upper left where the current branch is listed, change the branch to your feature-in-progress-branch. Open the files that you have worked on and check to make sure they include your changes.

**16. Create a pull request.**

Navigate your browser to https://github.com/prysmaticlabs/prysm and click on the new pull request button. In the “base” box on the left, leave the default selection “base master”, the branch that you want your changes to be applied to. In the “compare” box on the right, select feature-in-progress-branch, the branch containing the changes you want to apply. You will then be asked to answer a few questions about your pull request. After you complete the questionnaire, the pull request will appear in the list of pull requests at https://github.com/prysmaticlabs/prysm/pulls.

**17. Respond to comments by Core Contributors.**

Core Contributors may ask questions and request that you make edits. If you set notifications at the top of the page to “not watching,” you will still be notified by email whenever someone comments on the page of a pull request you have created. If you are asked to modify your pull request, repeat steps 8 through 15, then leave a comment to notify the Core Contributors that the pull request is ready for further review.

**18. If the number of commits becomes excessive, you may be asked to squash your commits.**

 You can do this with an interactive rebase. Start by running the following command to determine the commit that is the base of your branch...

```
$ git merge-base feature-in-progress-branch prysm/master
```

**19. The previous command will return a commit-hash that you should use in the following command.**

```
$ git rebase -i commit-hash
```

Your text editor will open with a file that lists the commits in your branch with the word pick in front of each branch such as the following …

```
pick 	hash	do some work
pick 	hash 	fix a bug
pick 	hash 	add a feature
```

Replace the word pick with the word “squash” for every line but the first, so you end with ….

```
pick    hash	do some work
squash  hash 	fix a bug
squash  hash 	add a feature
```

Save and close the file, then a commit command will appear in the terminal that squashes the smaller commits into one. Check to be sure the commit message accurately reflects your changes and then hit enter to execute it.

**20. Update your pull request with the following command.**

```
$ git push myrepo feature-in-progress-branch -f
```

**21.  Finally, again leave a comment to the Core Contributors on the pull request to let them know that the pull request has been updated.**

## Contributor Responsibilities

We consider two types of contributions to our repo and categorize them as follows:

### Part-Time Contributors

Anyone can become a part-time contributor and help out on implementing Ethereum consensus. The responsibilities of a part-time contributor include:

-   Engaging in Gitter conversations, asking the questions on how to begin contributing to the project
-   Opening up GitHub issues to express interest in code to implement
-   Opening up PRs referencing any open issue in the repo. PRs should include:
    -   Detailed context of what would be required for merge
    -   Tests that are consistent with how other tests are written in our implementation
-   Proper labels, milestones, and projects (see other closed PRs for reference)
-   Follow up on open PRs
    -   Have an estimated timeframe to completion and let the core contributors know if a PR will take longer than expected

### Core Contributors

Core contributors are remote contractors of Prysmatic Labs, LLC. and are considered critical team members of our organization. Core devs have all the responsibilities of part-time contributors plus the majority of the following:

-   Stay up to date on the latest beacon chain specification
-   Monitor GitHub issues and PR’s to make sure owner, labels, descriptions are correct
-   Formulate independent ideas, suggest new work to do, point out improvements to existing approaches
-   Participate in code review, ensure code quality is excellent, and ensure high code coverage
-   Help with social media presence, write bi-weekly development update
-   Represent Prysmatic Labs at events to help spread the word on scalability research and solutions

We love working with people that are autonomous, bring independent thoughts to the team, and are excited for their work! We believe in a merit-based approach to becoming a core contributor, and any part-time contributor that puts in the time, work, and drive can become a core member of our team.

![eth](https://steemitimages.com/DQmV1NASyCJYusDjY1WCvpoWiXh32HyumQHFQhY8zYZ6WDH/source.gif)
