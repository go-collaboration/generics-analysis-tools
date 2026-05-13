## Usage

Place a directory called "repos" within the language directory (e.g. "go").
It should contain the repositories to analyze as cloned by [clonya](https://github.com/go-collaboration/clonya).
Of course, symlinking to that directory is possible as well.

Make sure to build the analyzer tool for the language you are trying to analyze (see instructions below).
These tools are invoked using the `runall.py` script. This script has the following requirements:
- [Python](https://www.python.org/)
- [libPyshell](https://github.com/skogsbaer/libPyshell) needs to be available in the Python environment
- [cloc](https://github.com/aldanial/cloc)
- [git](https://git-scm.com/) (only when analyzing the history of some repositories)

Increase the stack size before running the analysis.
I'm not sure what the upper limit is, but the following worked for our use case:

    ulimit -s 131072

Run the analysis like this:

    python runall.py go

Change "go" to the the language of your choice.
When analyzing the history of repositories is desired, add "history" as a second argument.

Error messages will be printed to stderr. stdout can be redirected to a file and contains the analysis results as CSV.

### Go

Build the analyzer for Go:

    cd go
    go build

### Java

Build the analyer for Java:

    cd java/analyzer
    ./gradlew shadowJar

### Rust

To build the analyzer for Rust, a nightly toolchain is required.
As this is not always available, the toolchain is installed into a Docker image.

    cd rust
    sh build.sh

If you have a nightly toolchain available, you can probably also use `cargo build --release` and update `runall.py` accordingly.
