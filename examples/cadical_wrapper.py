import subprocess
import sys
import re
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

def run_cadical():
    global p
    f = open(sys.argv[2], "w")
    command = "./testing/cadical"
    p = subprocess.Popen([command, sys.argv[1]], stdout=f)

    p.wait()
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
    f.write("SPLITS DATA\n")
    d = dict()
    d["time"] = time
    d["blocked"] = blocked
    f.write(f"{d}\n".replace("'", '"'))


if __name__ == "__main__":
    run_cadical()
