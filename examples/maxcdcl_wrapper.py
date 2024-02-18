#!/usr/bin/python3

# This might not work out of the box because I have a slightly
# modified version of maxcdcl on my local machine.
#!/usr/bin/python3
import subprocess
import sys
import signal


def get_time(output: str) -> float:
    relevant = output.split("===========================")[-1]
    s_ticks_idx = relevant.find("CPU time")
    num_ticks = float(relevant[s_ticks_idx + 24 : s_ticks_idx + 33].split("s")[0])
    return num_ticks


p = None


def signal_handler(sig, frame):
    p.kill()
    exit(1)


signal.signal(signal.SIGTERM, signal_handler)


def run_maxcdcl():
    global p
    command = "maxcdcl"
    p = subprocess.Popen([command, sys.argv[1], sys.argv[2]])

    p.wait()

    # in case we crash
    if p.returncode != 10 and p.returncode != 20:
        f = open(sys.argv[2], "r")
        f.write("SPLITS DATA\n")
        d = dict()
        d["time"] = 9999999999
        f.write(f"{d}\n".replace("'", '"'))
        f.close()
        exit(1)

    f = open(sys.argv[2], "r")

    output = f.read()
    f.close()

    f = open(sys.argv[2], "a")

    # This is the important part:
    # Printing "SPLITS DATA" and then
    # the output in json format
    f.write("SPLITS DATA\n")
    d = dict()
    d["time"] = get_time(output)

    f.write(f"{d}\n".replace("'", '"'))


if __name__ == "__main__":
    run_maxcdcl()
