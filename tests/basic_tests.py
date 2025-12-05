import subprocess

def main():
    test_list = [
        "deposit",
        "withdrawal",
        "resolve",
        "chargeback",
    ]
    # for each test
    for test in test_list:
        print(f"Running test: {test}")
        # prepend the data directory
        input_filepath = "tests/data/" + test + "/input.csv"
        expected_output_filepath = "tests/data/" + test + "/expected_output.csv"
        # run the Rust code and collect stdout
        output = subprocess.check_output(['cargo', 'run', '--', input_filepath])
        # split into lines, then split each line by comma, then remove whitespace from elements
        output = [[x.strip() for x in line.split(',')] for line in output.decode("utf-8").splitlines()]
        # read into lines, then split each line by comma, then remove whitespace from elements
        with open(expected_output_filepath, mode="r", newline="") as f:
            expected_output = [[x.strip() for x in line.split(',')] for line in f.readlines()]
        # check output matches expected output (if there is a difference in length, tests will error out)
        success = True
        for i in range(0, max(len(output), len(expected_output))):
            # This doesn't check for capitalisation or for floating point decimal places
            if output[i] != expected_output[i]:
                success = False
                print(f"Difference found on line {i}")
                print(f"Output:   {output[i]}")
                print(f"Expected: {expected_output[i]}")
        
        if success:
            print("Test passed")
        else:
            print("Test failed")


if __name__ == "__main__":
    main()