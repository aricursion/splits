#!/usr/bin/python3
import subprocess
import sys
import re


def get_time(cadical_output):
    lookfor = "total process time since initialization:"
    anchor1 = cadical_output.find(lookfor)

    time = float(cadical_output[anchor1 + len(lookfor) :].split("seconds")[0].strip())
    return time


def get_conflicts(cadical_output):
    lookfor1 = "--- [ statistics ] -----------------------"
    relevant = cadical_output.split(lookfor1)[-1]

    lookfor2 = "conflicts:"
    lookfor3 = "per second"
    nums = re.sub(" +", " ", relevant.split(lookfor2)[1].split(lookfor3)[0]).strip()
    return float(nums.split(" ")[0])


def get_blocked(cadical_output):
    lookfor1 = "--- [ statistics ] -----------------------"
    relevant = cadical_output.split(lookfor1)[-1]

    lookfor2 = "blocked:"
    lookfor3 = "of irredundant"
    nums = re.sub(" +", " ", relevant.split(lookfor2)[1].split(lookfor3)[0]).strip()
    return float(nums.split(" ")[0])


def get_decisions(cadical_output):
    lookfor1 = "--- [ statistics ] -----------------------"
    relevant = cadical_output.split(lookfor1)[-1]

    lookfor2 = "decisions:"
    lookfor3 = "per second"
    nums = re.sub(" +", " ", relevant.split(lookfor2)[1].split(lookfor3)[0]).strip()
    return float(nums.split(" ")[0])


def get_learned(cadical_output):
    lookfor1 = "--- [ statistics ] -----------------------"
    relevant = cadical_output.split(lookfor1)[-1]

    lookfor2 = "c learned:"
    lookfor3 = "per conflict"
    nums = re.sub(" +", " ", relevant.split(lookfor2)[1].split(lookfor3)[0]).strip()
    return float(nums.split(" ")[0])


def get_props(cadical_output):
    lookfor1 = "--- [ statistics ] -----------------------"
    relevant = cadical_output.split(lookfor1)[-1]

    lookfor2 = "propagations:"
    lookfor3 = "per second"
    nums = re.sub(" +", " ", relevant.split(lookfor2)[1].split(lookfor3)[0]).strip()
    return float(nums.split(" ")[0])


def get_subsumed(cadical_output):
    lookfor1 = "--- [ statistics ] -----------------------"
    relevant = cadical_output.split(lookfor1)[-1]

    lookfor2 = "subsumed:"
    lookfor3 = "of all clauses"
    nums = re.sub(" +", " ", relevant.split(lookfor2)[1].split(lookfor3)[0]).strip()
    return float(nums.split(" ")[0])


def run_cadical():
    f = open(sys.argv[2], "w")
    command = "./testing/cadical"
    p = subprocess.Popen([command, sys.argv[1]], stdout=f)

    p.wait()
    f.close()

    f = open(sys.argv[2], "r")

    output = f.read()
    time = get_time(output)
    blocked = get_blocked(output)

    f.close()
    f = open(sys.argv[2], "a")

    # This is the important part:
    # Printing "SPLITS DATA" and then
    # the output in json format
    f.write("SPLITS DATA\n")
    d = dict()
    d["time"] = time
    d["blocked"] = blocked
    d["subsumed"] = get_subsumed(output)
    d["propogations"] = get_props(output)
    d["learned"] = get_learned(output)
    d["decisions"] = get_decisions(output)
    d["conflicts"] = get_conflicts(output)

    f.write(f"{d}\n".replace("'", '"'))


if __name__ == "__main__":
    run_cadical()
