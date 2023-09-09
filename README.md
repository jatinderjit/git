# Git

Yet another Git implementation

## Concepts

- The core of Git is a simple key-value data store. You can insert any kind of
  content into a Git repository, for which Git will hand you back a key, that
  you can use later to retrieve that content.
- An important property is that the same value will always result in the same key.
- The keys here are `sha1` hashes of the content. Branch names, tags, etc could
  also be used as the "keys", as they are simply references to the hashes.
  - `sha256` is also _experimentally_ supported by Git.
- There are (at least?) three kind of objects: blob, tree and commit, and all
  these "values" are stored in `.git/objects/`.
- Use `hash-object` to compute the hash of an object, and `-w` flag to save the
  object as well. This hash can be considered as the key.
- Use `cat-file` to print the contents of an object referenced by commit hash or
  name (HEAD / branch name). This is equivalent to retrieving the value for the
  key (object name).

```console
$ git init
$ echo 'hello' > test.txt
$ git hash-object test.txt
ce013625030ba8dba906f756967f9e9ca394464a
$ find .git/objects -type f
.git/objects/ce/013625030ba8dba906f756967f9e9ca394464a
$ git cat-file -p ce01362
hello
$ git cat-file -t ce01362
blob
```

### Objects

The object format:

```txt
<object-type><space><content-length><null-byte><content>
```

1. The object file starts with the type of object, followed by a single space.
1. This is followed by the content size - length of content (in bytes), until a
   null byte is found.
1. The actual content starts after the null byte.
1. SHA1 of this final sequence of bytes will be computed, which will be used as
   the key.
1. This byte sequence will be compressed (using zlib) and stored in the objects
   directory.

The text from the above example will be stored as `blob 6\x00hello\n`

```python
>>> import hashlib
>>> obj = b"blob 6\x00hello\n"
>>> print(hashlib.sha1(obj).hexdigest())
ce013625030ba8dba906f756967f9e9ca394464a  # same as above
```

Note: the hash is computed before the compression. Otherwise, a change in
compression algorithm could result in the same content producing different
hashes.

#### Blob

- An arbitrary sequence of bytes
- Filenames don't matter. Git just stores the content of a file as a blob. Extra
  information is managed by the tree object.

#### Tree

- Tree objects in Git form a [Merkle Tree](https://en.wikipedia.org/wiki/Merkle_tree).
- Each directory in a repository forms a tree. Each tree object stores
  references to other blobs (in case of files) or other trees (in case of
  directories), in a lexicographically order.

A sample tree node:

```console
$ git cat-file -p <tree-object>
100644 blob 9eddd609a9d0f894728ef29362a0ae0dfdd2b63b    .gitignore
100644 blob d80fafe1c5c1f60cfd51ebfbd83dd2936dfb5491    Cargo.toml
100644 blob 93f841163fef854ed074690c01d2f72f8352e50c    LICENSE
100644 blob f59fdb47223e85578c82867a36abbe9196dfb17c    README.md
040000 tree 162ba22ccc651092fc5c6deebe4afcf17629fb2e    src
```

#### Commit

- A commit records changes to the repository. In the object format, the fields
  in a commit are separated by newlines.
- Each commit typically stores (not an exhaustive list):
  - `tree` object hash that stores the current state of the index.
  - `parent` commit(s). There is generally one parent commit, but there could be
    zero parents (in case of root commit), or two parents (merge commits) as
    well.
  - `author`, `timestamp`
  - `commit message`

## Progress

- [ ] [Plumbing](https://git-scm.com/book/en/v2/Git-Internals-Plumbing-and-Porcelain)
  - [x] `init`: initializes an empty git repository
  - [x] `cat-file`: provides content/type/size information for repository objects
  - [x] `hash-object`: computes content-hash and (optionally) create a blob
  - [x] `ls-tree`: displays contents of the tree (or a commit's tree) object
  - [ ] `write-tree`: creates a tree object from the current index
  - [ ] `commit-tree`: creates a commit object for the tree
  - [ ] `update-ref`: changes object name (branch/commit) stored in a ref (HEAD)
- [ ] Porcelain
  - [ ] `branch`: create/rename/delete branches
  - [ ] `switch`: change active branch (scan for diffs, and abort in case of conflicts)
  - [ ] `add`: stages the changes (add to index)
  - [ ] `restore`: resets changes as per the working tree
  - [ ] `commit`: creates a tree and commit object from the current index
  - [ ] `log`: shows commit history
  - [ ] `cherry-pick`: re-apply changes from existing commits (same/different branch)
  - [ ] `merge`: handle 3-way merge
- [ ] Client-server
  - [ ] `clone` a repository
  - [ ] `pull` a repository
  - [ ] `push` a repository

Note: `git checkout` won't be supported. We'll use `switch` (branch) or
`restore` (working tree) instead.

## Resources

- [Git Internals](https://git-scm.com/book/en/v2/Git-Internals-Plumbing-and-Porcelain)
