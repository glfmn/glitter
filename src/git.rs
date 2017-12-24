use git2::Repository;


/// Stats which the interpreter uses to populate the gist expression
#[derive(Debug, PartialEq, Eq, Default, Clone)]
pub struct Stats {
    /// Number of untracked files which are new to the repository
    pub untracked: u16,
    /// Number of files to be added
    pub added_staged: u16,
    /// Number of modified files which have not yet been staged
    pub modified: u16,
    /// Number of staged changes to files
    pub modified_staged: u16,
    /// Number of renamed files
    pub renamed: u16,
    /// Number of deleted files
    pub deleted: u16,
    /// Number of staged deletions
    pub deleted_staged: u16,
    /// Number of commits ahead of the upstream branch
    pub ahead: u16,
    /// Number of commits behind the upstream branch
    pub behind: u16,
    /// Number of unresolved conflicts in the repository
    pub conflicts: u16,
    /// Number of stashes on the current branch
    pub stashes: u16,
    /// The branch name or other stats of the HEAD pointer
    pub branch: String,
    /// The of the upstream branch
    pub remote: String,
}


impl Stats {

}
