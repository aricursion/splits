import subprocess
import sys
import signal


def parse_metric(output: str) -> float:
    relevant = output.split("===========================")[-1]
    s_ticks_idx = relevant.find("CPU time")
    num_ticks = float(relevant[s_ticks_idx + 24 : s_ticks_idx + 33].split("s")[0])
    return num_ticks


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
    log_file = open(sys.argv[2], "w")

    command = "./testing/maxcdcl"
    p = subprocess.Popen([command, sys.argv[1]], stdout=log_file)

    p.wait()
    # If the process completes, we should block SIGTERM so we can
    # finish writing the file and exit normally
    signal.pthread_sigmask(signal.SIG_BLOCK, [signal.SIGTERM])
    log_file.close()

    log_file = open(sys.argv[2], "r")

    output = log_file.read()

    log_file.close()
    log_file = open(sys.argv[2], "a")

    # This is the important part:
    # Printing "SPLITS DATA" and then
    # the output in json format
    log_file.write("SPLITS DATA\n")
    d = dict()

    # You can add other metrics in the exact same way
    metric_name = "time"

    d[metric_name] = parse_metric(output)

    log_file.write(f"{d}\n".replace("'", '"'))


if __name__ == "__main__":
    run_cadical()
