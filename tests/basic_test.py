import csv
import random
from pathlib import Path

TYPES = ["deposit", "withdrawal", "dispute", "resolve", "chargeback"]

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
    Path("tests/data/tmp").mkdir(exist_ok=True)

    # Generate a few test files
    generate_csv("tests/data/tmp/sample_small.csv", 100)

if __name__ == "__main__":
    main()