# Maven Hasher
A simple rust based program for generating missing hashes for
a Maven repository.

```
USAGE:
    maven-hasher [FLAGS] --repo <FOLDER>

FLAGS:
        --dry-run    Doesn't hash things, just tells you what it will do. Implies verbose.
    -h, --help       Prints help information
    -v, --verbose    Tells you what its hashing.
    -V, --version    Prints version information

OPTIONS:
    -r, --repo <FOLDER>    The folder on disk representing the maven repository.

```

