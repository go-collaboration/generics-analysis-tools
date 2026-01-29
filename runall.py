import json
import os
import sys
from shell import *


node_path = "" # you likely need to manually set your node path here. i was too lazy to do something automatic, just for running this once or twice...


def commit_list(path):
    old_pwd = pwd()
    cd(path)
    cmd = ["git", "log", "--format=%H %ct"]
    res = run(cmd, captureStdout=True, onError="raise")
    commits = list(map(lambda l: l.split(" "), res.stdout.splitlines()))
    commits = list(map(lambda l: (l[0], int(l[1])), commits))
    selected_commits = []
    i = 0
    while i < len(commits):
        selected_commits.append(commits[i])
        while i < len(commits) and commits[i][1] > selected_commits[-1][1] - 90 * 86400:
            i += 1
    cd(old_pwd)
    return selected_commits


def checkout(path, hash):
    old_pwd = pwd()
    cd(path)
    cmd = ["git", "checkout", "-f", hash]
    run(cmd, captureStdout=True, captureStderr=True, onError="raise")
    cd(old_pwd)


def find_files(repo):
    paths = []
    for root, dirs, files in os.walk(repo, followlinks=False):
        for file in files:
            if os.path.islink(os.path.join(root, file)):
                continue
            if lang == "crystal" and file.endswith(".cr"):
                paths.append(os.path.join(root, file))
            elif lang == "csharp" and file.endswith(".cs"):
                paths.append(os.path.join(root, file))
            elif lang == "java" and file.endswith(".java"):
                paths.append(os.path.join(root, file))
            elif lang == "go" and file.endswith(".go"):
                paths.append(os.path.join(root, file))
            elif lang == "typescript" and file.endswith(".ts"):
                paths.append(os.path.join(root, file))
            elif lang == "rust" and file.endswith(".rs"):
                path = os.path.join(root, file)
                path = path.removeprefix(repo + "/")
                paths.append(path)
    return paths


def cloc(files):
    sum = 0
    for part in [files[i:i+16000] for i in range(0, len(files), 16000)]:
        cmd = ["cloc", "--json"]
        cmd.extend(part)
        res = run(cmd, captureStdout=True, captureStderr=True, onError="raise")
        sum += int(json.loads(res.stdout)["SUM"]["code"])
    return sum


def run_split(base_cmd, paths, nums_len):
    nums = [0] * nums_len
    if len(paths) > 0:
        path_parts = [paths[i:i+16000] for i in range(0, len(paths), 16000)]

        for part in path_parts:
            cmd = base_cmd.copy()
            cmd.extend(part)
            try:
                res = run(cmd, captureStdout=True, onError="raise", env={"NODE_PATH": node_path})
            except OSError:
                print("error! cmd len:", len(cmd))
                exit(1)
            cur_nums = list(map(lambda x: int(x), res.stdout.split(",")))
            for i in range(len(nums)):
                nums[i] += cur_nums[i]
    print(",".join(map(lambda x: str(x), nums)))


lang = sys.argv[1]
with_history = len(sys.argv) > 2 and sys.argv[2] == "history"
for repo in ls(pjoin(dirname(__file__), lang, "repos")):
    name = basename(repo)
    name = name.split("#")
    name = name[0] + "/" + name[1]
    nums_len = 4
    if lang == "crystal":
        cmd = [pjoin(dirname(__file__), lang, "bin/analyzer")]
    elif lang == "csharp":
        cmd = [pjoin(dirname(__file__), lang, "analyzer/bin/Release/net9.0/Analyzer")]
        nums_len = 5
    elif lang == "java":
        cmd = ["java", "-Xss4m","-jar", pjoin(dirname(__file__), lang, "analyzer/app/build/libs/analyzer.jar")]
        nums_len = 6
    elif lang == "go":
        cmd = [pjoin(dirname(__file__), lang, "analyzer")]
        nums_len = 6
    elif lang == "typescript":
        cmd = ["node", "--stack-size=131072", "-r", "ts-node/register", pjoin(dirname(__file__), lang, "analyzer/analyzer.ts")]
        nums_len = 7
    elif lang == "rust":
        cmd = ["docker", "run", "--rm", "--net=host", f"-v{repo}:/proj", "--workdir", "/analyzer", f"-v{pjoin(dirname(__file__), lang, "analyzer")}:/analyzer", "rust-nightly:2025-12-18", "/root/.cargo/bin/cargo", "run", "--quiet", "--release", "--", "/proj"]

    if not with_history:
        paths = find_files(repo)
        print(f"{name},{cloc(paths)},", end="")
        run_split(cmd, paths, nums_len)
    else:
        commits = commit_list(repo)
        for i, commit in enumerate(commits):
            if i > 0:
                checkout(repo, commit[0])
            paths = find_files(repo)
            print(f"{name},{commit[1]},{cloc(paths)},", end="")
            run_split(cmd, paths, nums_len)
            if commit[1] < 1639738800: # 2021-12-17 (a few months before the Go 1.18 release with generics: https://www.youtube.com/watch?v=Pa_e9EeCdy8)
                break
        checkout(repo, commits[0][0])
