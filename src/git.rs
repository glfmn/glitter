use git2;
use std::ops::{AddAssign, BitAnd};
use std::fmt::Write;


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
    /// Populate stats with the status of the given repository
    pub fn new(repo: &mut git2::Repository) -> Result<Stats, git2::Error> {
        let mut st: Stats = Default::default();

        st.read_branch(repo);

        let mut opts = git2::StatusOptions::new();

        opts.include_untracked(true)
            .recurse_untracked_dirs(true);

        {
            let statuses = repo.statuses(Some(&mut opts))?;

            for status in statuses.iter() {
                let flags = status.status();

                if check(flags, git2::STATUS_WT_NEW) {
                    st.untracked += 1;
                }
                if check(flags, git2::STATUS_INDEX_NEW) {
                    st.added_staged += 1;
                }
                if check(flags, git2::STATUS_WT_MODIFIED) {
                    st.modified += 1;
                }
                if check(flags, git2::STATUS_INDEX_MODIFIED) {
                    st.modified_staged += 1;
                }
                if check(flags, git2::STATUS_INDEX_RENAMED) {
                    st.renamed += 1;
                }
                if check(flags, git2::STATUS_WT_DELETED) {
                    st.deleted += 1;
                }
                if check(flags, git2::STATUS_INDEX_DELETED) {
                    st.deleted_staged += 1;
                }
                if check(flags, git2::STATUS_CONFLICTED) {
                    st.conflicts += 1;
                }
            }
        }

        repo.stash_foreach(|_, &_, &_,| {
            st.stashes += 1;
            true
        })?;

        Ok(st)
    }

    /// Read the branch-name of the repository
    ///
    /// If in detached head, grab the first few characters of the commit ID if possible, otherwise
    /// simply provide HEAD as the branch name.  This is to mimic the behaviour of `git status`.
    fn read_branch(&mut self, repo: &git2::Repository) {
        self.branch = match repo.head() {
            Ok(head) => {
                if let Some(name) = head.name() {
                    // try to use first 8 characters or so of the ID in detached HEAD
                    if name == "HEAD" {
                        if let Ok(commit) = head.peel_to_commit() {
                            let mut s = String::new();
                            for byte in &commit.id().as_bytes()[..4] {
                                write!(&mut s, "{:x}", byte).unwrap();
                            }
                            s
                        } else {
                            "HEAD".to_string()
                        }
                    // Grab the branch from the reference
                    } else {
                        name.split("/").last().unwrap_or("").to_string()
                    }
                } else {
                    "HEAD".to_string()
                }
            },
            Err(ref err) if err.code() == git2::ErrorCode::BareRepo => "master".to_string(),
            Err(_) if repo.is_empty().unwrap_or(false) => "master".to_string(),
            Err(_) => "HEAD".to_string(),
        };
    }
}

impl AddAssign for Stats {
    fn add_assign(&mut self, rhs: Self) {
        self.untracked += rhs.untracked;
        self.added_staged += rhs.added_staged;
        self.modified += rhs.modified;
        self.modified_staged += rhs.modified_staged;
        self.renamed += rhs.renamed;
        self.deleted += rhs.deleted;
        self.deleted_staged += rhs.deleted_staged;
        self.ahead += rhs.ahead;
        self.behind += rhs.behind;
        self.conflicts += rhs.conflicts;
        self.stashes += rhs.stashes;
    }
}

/// Check the bits of a flag against the value to see if they are set
#[inline]
fn check<B>(val: B, flag: B) -> bool
    where B: BitAnd<Output=B> + PartialEq + Copy {
    val & flag == flag
}