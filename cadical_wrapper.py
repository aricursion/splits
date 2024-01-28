import subprocess
import sys
import re
import os
import signal


def get_time(cadical_output):
    lookfor = "total process time since initialization:"
    anchor1 = cadical_output.find(lookfor)

    time = float(cadical_output[anchor1 + len(lookfor) :].split("seconds")[0].strip())
    return time


def get_blocked(cadical_output):
    lookfor1 = "--- [ statistics ] -----------------------"
    relevant = cadical_output.split(lookfor1)[-1]

    lookfor2 = "conflicts:"
    lookfor3 = "per second"
    nums = re.sub(" +", " ", relevant.split(lookfor2)[1].split(lookfor3)[0]).strip()
    return float(nums.split(" ")[0])


def term_handler(sig, frame):
    # Force all the children to be killed
    # This will kill other cadicals running
    # But hopefully this is not an issue.
    subprocess.run(["pkill", "cadical"])
    exit(0)

# Install the signal handler to recieve 
# a SIGTERM
signal.signal(signal.SIGTERM, term_handler)


def run_cadical():
    f = open(sys.argv[2], "w")
    subprocess.run(["./testing/cadical", sys.argv[1]], stdout=f, preexec_fn=os.setsid)

    # If the process completes, we should block SIGTERM so we can
    # finish writing the file and exit normally
    signal.pthread_sigmask(signal.SIG_BLOCK, [signal.SIGTERM])
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
    f.write("SPLITS DATA")
    d = dict()
    d["time"] = time
    d["blocked"] = blocked
    f.write(f"{d}")


if __name__ == "__main__":
    run_cadical()
