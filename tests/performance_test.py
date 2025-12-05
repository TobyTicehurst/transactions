import csv
import random
import subprocess
from pathlib import Path

TYPES = ["deposit", "withdrawal"]

def generate_csv(filename: str, num_rows: int):
    """Generate a CSV file with test transaction data."""
    with open(filename, mode="w", newline="") as f:
        writer = csv.writer(f)
        writer.writerow(["type", "client", "tx", "amount"])

        tx_id = 1
        for _ in range(num_rows):
            row_type = random.choice(TYPES)
            client_id = random.randint(1, 50)
            amount = round(random.uniform(0.01, 1000.0), 4)
            writer.writerow([row_type, client_id, tx_id, amount])
            tx_id += 1

def main():
    test_dir = "tests/data/tmp/"
    Path(test_dir).mkdir(exist_ok=True)

    input_filepath = test_dir + "sample_large.csv"
    generate_csv(input_filepath, 10_000_000)
    subprocess.check_output(['cargo', 'run', '--release', '--', input_filepath])

if __name__ == "__main__":
    main()