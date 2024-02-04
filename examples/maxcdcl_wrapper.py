#!/usr/bin/python3
import subprocess
import sys
import signal


def parse_std_out_time(out):
    relevant = out.split("===========================")[-1]
    s_time_idx = relevant.find("CPU time")
    time = float(relevant[s_time_idx + 24 : s_time_idx + 33].split("s")[0])
    return time


def parse_std_out_ticks(out):
    # if unsat, result output does not include ticks
    if "UNSATISFIABLE" in out:
        return -1
    relevant = out.split("===========================")[-1]
    ticks_idx = relevant.find("s_ticks")
    ticks = float(relevant[ticks_idx : ticks_idx + 30].split(" ")[1].split("\n")[0])
    return ticks


def parse_std_out_conflicts(out):
    relevant = out.split("===========================")[-1]
    conflicts_idx = relevant.find("conflicts")
    conflicts = float(
        relevant[conflicts_idx + 24 : conflicts_idx + 33].split("(")[0].strip()
    )
    return conflicts


def parse_std_out_props(out):
    relevant = out.split("===========================")[-1]
    props_idx = relevant.find("propagations")
    props = float(relevant[props_idx + 24 : props_idx + 35].split("(")[0].strip())
    return props


def term_handler(sig, frame):
    p.kill()

    # Write Splits data. This is optional
    # in the sense that the SPLITS tool
    # knows which children terminate
    # early without reading the logs
    f = open(sys.argv[2], "a")
    f.write("SPLITS DATA\n")
    f.write("Terminated\n")
    exit(0)


# Install the signal handler to recieve
# a SIGTERM
signal.signal(signal.SIGTERM, term_handler)

p = None


# catchable_sigs = set(signal.Signals) - {signal.SIGKILL, signal.SIGSTOP}
# for sig in catchable_sigs:
#     signal.signal(sig, term_handler)  # Substitute handler of choice for `print`


def run_maxcdcl():
    global p

    command = "./testing/maxcdcl"
    p = subprocess.Popen([command, sys.argv[1], sys.argv[2]])

    p.wait()
    # If the process completes, we should block SIGTERM so we can
    # finish writing the file and exit normally
    signal.pthread_sigmask(signal.SIG_BLOCK, [signal.SIGTERM])
    log_file = open(sys.argv[2], "r")
    output = log_file.read()

    log_file.close()
    log_file = open(sys.argv[2], "a")

    # This is the important part:
    # Printing "SPLITS DATA" and then
    # the output in json format
    log_file.write("SPLITS DATA\n")
    d = dict()

    d["conflicts"] = parse_std_out_conflicts(output)
    d["propts"] = parse_std_out_props(output)
    d["time"] = parse_std_out_time(output)
    d["ticks"] = parse_std_out_ticks(output)

    log_file.write(f"{d}\n".replace("'", '"'))


if __name__ == "__main__":
    run_maxcdcl()
