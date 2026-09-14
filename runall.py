import json
import os
import sys
from shell import *


node_path = "" # you likely need to manually set your node path here. i was too lazy to do something automatic, just for running this once or twice...


def commit_list(path):
    old_pwd = pwd()
    cd(path)
    cmd = ["git", "log", "--format=%H %ct %at"]
    res = run(cmd, captureStdout=True, onError="raise")
    commits = list(map(lambda l: l.split(" "), res.stdout.splitlines()))
    commits = list(map(lambda l: (l[0], int(l[1]), int(l[2])), commits))
    selected_commits = []
    i = 0
    while i < len(commits):
        selected_commits.append(commits[i])
        while i < len(commits) and commits[i][1] > selected_commits[-1][1] - 90 * 86400:
            i += 1
            if i < len(commits) and "BlueJ-Greenfoot" in path and commits[i][1] < 1490626634:
                commits[i] = (commits[i][0], commits[i][2], commits[i][2])
    cd(old_pwd)
    return selected_commits


def checkout(path, hash):
    old_pwd = pwd()
    cd(path)
    cmd = ["git", "checkout", "-f", hash]
    run(cmd, captureStdout=True, captureStderr=True, onError="raise")
    cd(old_pwd)


def strip_repo_prefix(repo, paths):
    return list(map(lambda path: path.removeprefix(repo + "/"), paths))


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
                paths.append(os.path.join(root, file))
    analyzer_args = paths
    if lang == "rust":
        analyzer_args = strip_repo_prefix(repo, paths)
    return paths, analyzer_args


def cloc(files):
    sum = 0
    for part in [files[i:i+6000] for i in range(0, len(files), 6000)]:
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
            cur_nums = list(map(lambda x: list(map(lambda y: int(y), x.split(":"))) if ":" in x else int(x), res.stdout.split(",")))
            for i in range(len(nums)):
                if type(cur_nums[i]) is list:
                    if nums[i] == 0:
                        nums[i] = []
                    for j, num in enumerate(cur_nums[i]):
                        while j >= len(nums[i]):
                            nums[i].append(0)
                        nums[i][j] += num
                else:
                    nums[i] += cur_nums[i]
    print(",".join(map(lambda x: ":".join(map(lambda y: str(y), x)) if type(x) is list else str(x), nums)))


lang = sys.argv[1]
with_history = len(sys.argv) > 2 and sys.argv[2] == "history"
for repo_num, repo in enumerate(ls(pjoin(dirname(__file__), lang, "repos"))):
    name = basename(repo)
    name = name.split("#")
    name = name[0] + "/" + name[1]
    if lang == "crystal":
        cmd = [pjoin(dirname(__file__), lang, "bin/analyzer")]
        analyzer_columns = ["parse_errors", "generic_types", "non_generic_types", "generic_functions", "non_generic_functions"]
    elif lang == "csharp":
        cmd = [pjoin(dirname(__file__), lang, "analyzer/bin/Release/net9.0/Analyzer")]
        analyzer_columns = ["parse_errors", "generic_types", "non_generic_types", "generic_functions", "non_generic_functions", "casts", "is_patterns"]
    elif lang == "java":
        cmd = ["java", "-Xss4m","-jar", pjoin(dirname(__file__), lang, "analyzer/app/build/libs/analyzer.jar")]
        analyzer_columns = ["parse_errors", "generic_types", "non_generic_types", "generic_classes", "generic_interfaces", "generic_functions", "non_generic_functions", "generic_non_static_methods", "generic_static_methods", "casts", "instance_ofs", "type_parameter_count", "type_parameter_count_classes", "type_parameter_count_interfaces", "type_parameter_count_non_static_methods", "type_parameter_count_static_methods", "non_trivial_type_bounds", "trivial_type_bounds", "non_trivial_type_bounds_classes", "trivial_type_bounds_classes", "non_trivial_type_bounds_interfaces", "trivial_type_bounds_interfaces", "non_trivial_type_bounds_non_static_methods", "trivial_type_bounds_non_static_methods", "non_trivial_type_bounds_static_methods", "trivial_type_bounds_static_methods"]
    elif lang == "go":
        cmd = [pjoin(dirname(__file__), lang, "analyzer")]
        analyzer_columns = ["parse_errors", "generic_types", "non_generic_types", "generic_structs", "generic_interfaces", "generic_type_aliases", "generic_other", "generic_functions", "non_generic_functions", "non_trivial_type_bounds", "trivial_type_bounds", "non_trivial_type_bounds_structs", "trivial_type_bounds_structs", "non_trivial_type_bounds_interfaces", "trivial_type_bounds_interfaces", "non_trivial_type_bounds_functions", "trivial_type_bounds_functions", "non_trivial_type_bounds_type_aliases", "trivial_type_bounds_type_aliases", "non_trivial_type_bounds_other", "trivial_type_bounds_other", "type_assertions", "type_switches", "type_parameter_count", "type_parameter_count_structs", "type_parameter_count_interfaces", "type_parameter_count_functions", "type_parameter_count_type_aliases", "type_parameter_count_other"]
    elif lang == "typescript":
        cmd = ["node", "--stack-size=131072", "-r", "ts-node/register", pjoin(dirname(__file__), lang, "analyzer/analyzer.ts")]
        analyzer_columns = ["parse_errors", "generic_types", "non_generic_types", "generic_functions", "non_generic_functions", "casts", "type_ofs", "instance_ofs"]
    elif lang == "rust":
        cmd = ["docker", "run", "--rm", "--net=host", f"-v{repo}:/proj", "--workdir", "/analyzer", f"-v{pjoin(dirname(__file__), lang, "analyzer")}:/analyzer", "rust-nightly:2026-09-14", "/root/.cargo/bin/cargo", "run", "--quiet", "--release", "--", "/proj"]
        analyzer_columns = ["parse_errors", "generic_types", "non_generic_types", "generic_structs", "generic_traits", "generic_type_aliases", "generic_other", "generic_functions", "non_generic_functions", "generic_functions_only", "generic_methods", "type_parameter_count", "type_parameter_count_structs", "type_parameter_count_traits", "type_parameter_count_functions", "type_parameter_count_methods", "type_parameter_count_type_aliases", "type_parameter_count_other", "non_trivial_type_bounds", "trivial_type_bounds", "non_trivial_type_bounds_structs", "trivial_type_bounds_structs", "non_trivial_type_bounds_traits", "trivial_type_bounds_traits", "non_trivial_type_bounds_functions", "trivial_type_bounds_functions", "non_trivial_type_bounds_methods", "trivial_type_bounds_methods", "non_trivial_type_bounds_type_aliases", "trivial_type_bounds_type_aliases", "non_trivial_type_bounds_other", "trivial_type_bounds_other"]

    if repo_num == 0:
        header = ["repository", "loc", "num_files"] + analyzer_columns
        if with_history:
            header.insert(1, "commit_time")
        print(",".join(header))

    if not with_history:
        paths, analyzer_args = find_files(repo)
        print(f"{name},{cloc(paths)},{len(paths)},", end="")
        run_split(cmd, analyzer_args, len(analyzer_columns))
    else:
        commits = commit_list(repo)
        for i, commit in enumerate(commits):
            if i > 0:
                checkout(repo, commit[0])
            paths, analyzer_args = find_files(repo)
            print(f"{name},{commit[1]},{cloc(paths)},{len(paths)},", end="")
            run_split(cmd, analyzer_args, len(analyzer_columns))
            history_start = 1639738800 # 2021-12-17 (a few months before the Go 1.18 release with generics: https://www.youtube.com/watch?v=Pa_e9EeCdy8)
            if commit[1] < history_start and lang != "java":
                break
        checkout(repo, commits[0][0])
