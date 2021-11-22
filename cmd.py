import json
import os
import subprocess
import sys
from io import StringIO

def defn_field(obj, field: str):
    try:
        f = obj[field]
        if f == None:
            return []
        else:
            return f
    except:
        return []

if __name__ == "__main__":
    arg = sys.argv[1]
    arg_list = arg.split()

    if len(arg_list) == 0:
        print(arg)
        exit(0)

    cmd = arg_list[0]

    defn_file_name = 'defs/' + cmd + '.json'

    with open(defn_file_name, 'r') as defn_file:
        defn_obj = json.load(defn_file)
        arg_2_complete = arg_list[-1]

        completions = defn_field(defn_obj, "options") + defn_field(defn_obj, "subcommands")
        c_str = "\n".join([ it["name"] for it in completions])



        p = subprocess.Popen(['fzf'], stdin=subprocess.PIPE, text=True, encoding="utf-8")
        p.communicate(input=c_str)
        p.wait()
        exit()