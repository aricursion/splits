#!/usr/bin/python3
import subprocess
import sys
import re
import signal


def get_time(cadical_output):
    lookfor = "total process time since initialization:"
    anchor1 = cadical_output.find(lookfor)

    time = float(cadical_output[anchor1 + len(lookfor) :].split("seconds")[0].strip())
    return time


def get_conflicts(cadical_output):
    lookfor2 = "conflicts:"
    lookfor3 = "per second"
    nums = re.sub(
        " +", " ", cadical_output.split(lookfor2)[1].split(lookfor3)[0]
    ).strip()
    return float(nums.split(" ")[0])


def get_blocked(cadical_output):
    lookfor2 = "blocked:"
    lookfor3 = "of irredundant"
    nums = re.sub(
        " +", " ", cadical_output.split(lookfor2)[1].split(lookfor3)[0]
    ).strip()
    return float(nums.split(" ")[0])


def get_decisions(cadical_output):
    lookfor2 = "decisions:"
    lookfor3 = "per second"
    nums = re.sub(
        " +", " ", cadical_output.split(lookfor2)[1].split(lookfor3)[0]
    ).strip()
    return float(nums.split(" ")[0])


def get_learned(cadical_output):
    lookfor2 = "c learned:"
    lookfor3 = "per conflict"
    nums = re.sub(
        " +", " ", cadical_output.split(lookfor2)[1].split(lookfor3)[0]
    ).strip()
    return float(nums.split(" ")[0])


def get_props(cadical_output):
    lookfor2 = "propagations:"
    lookfor3 = "per second"
    nums = re.sub(
        " +", " ", cadical_output.split(lookfor2)[1].split(lookfor3)[0]
    ).strip()
    return float(nums.split(" ")[0])


def get_subsumed(cadical_output):
    lookfor2 = "subsumed:"
    lookfor3 = "of all clauses"
    nums = re.sub(
        " +", " ", cadical_output.split(lookfor2)[1].split(lookfor3)[0]
    ).strip()
    return float(nums.split(" ")[0])


p = None


def signal_handler(sig, frame):
    p.kill()
    exit(1)


signal.signal(signal.SIGTERM, signal_handler)


def run_cadical():
    global p
    f = open(sys.argv[2], "w")
    command = "cadical"
    p = subprocess.Popen([command, "-v", sys.argv[1]], stdout=f)

    p.wait()
    f.close()

    f = open(sys.argv[2], "r")

    output = f.read()
    lookfor1 = "--- [ statistics ] -----------------------"
    relevant = output.split(lookfor1)[-1]
    f.close()

    f = open(sys.argv[2], "a")

    # This is the important part:
    # Printing "SPLITS DATA" and then
    # the output in json format
    f.write("SPLITS DATA\n")
    d = dict()
    d["time"] = get_time(relevant)
    d["blocked"] = get_blocked(relevant)
    d["subsumed"] = get_subsumed(relevant)
    d["propogations"] = get_props(relevant)
    d["learned"] = get_learned(relevant)
    d["decisions"] = get_decisions(relevant)
    d["conflicts"] = get_conflicts(relevant)

    f.write(f"{d}\n".replace("'", '"'))


if __name__ == "__main__":
    run_cadical()
