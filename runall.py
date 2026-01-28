import os
import sys
from shell import *


node_path = "" # you likely need to manually set your node path here. i was too lazy to do something automatic, just for running this once or twice...


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
for repo in ls(pjoin(dirname(__file__), lang, "repos")):
    name = basename(repo)
    name = name.split("#")
    name = name[0] + "/" + name[1]
    nums_len = 4
    print(name+",", end="")
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
    run_split(cmd, paths, nums_len)

