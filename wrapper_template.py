import subprocess
import sys
import signal


def parse_metric(output: str) -> float:
    pass


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

    command = # fill this in
    p = subprocess.Popen([command, sys.argv[1]], stdout=f)

    p.wait()
    # If the process completes, we should block SIGTERM so we can
    # finish writing the file and exit normally
    signal.pthread_sigmask(signal.SIG_BLOCK, [signal.SIGTERM])
    log_file.close()

    log_file = open(sys.argv[2], "r")

    output = log_file.read()
    metric = parse_metric(output)

    log_file.close()
    log_file = open(sys.argv[2], "a")

    # This is the important part:
    # Printing "SPLITS DATA" and then
    # the output in json format
    log_file.write("SPLITS DATA\n")
    d = dict()

    # You can add other metrics in the exact same way
    metric_name = # fill this in


    d[metric_name] = parse_metric(output)

    log_file.write(f"{d}\n")


if __name__ == "__main__":
    run_cadical()
