#!/bin/env python3

# Imports
import sys
import subprocess


# Print usage
def print_usage():
	print(f"Usage: {sys.argv[0]} <game-file> [output-dir]")


# Retrieves the arguments
def get_args():
	try:
		game_file = sys.argv[1]
	except:
		print_usage()
		exit(1)
	try:
		output_dir = sys.argv[2]
	except:
		output_dir = "./game"

	return (game_file, output_dir)


# Extracts the card table
def extract_card_table(game_file, output_file):
	with open(output_file, 'w') as f:
		args = ["cargo", "run", "--bin", "dcb-extract-card-table", "--", game_file]
		proc = subprocess.Popen(args, stdout=f)
		if proc.wait() != 0:
			print("Failed to extract card table")
			exit(1)


# Main
def main():
	# Get arguments
	(game_file, output_dir) = get_args()

	# If the game file isn't a `.bin`, exit
	if not game_file.lower().endswith('.bin'):
		print("The game file must bea `.bin` file")
		exit(1)

	# Extract the card and deck table
	extract_card_table(game_file, f"{output_dir}/card_table.json")

	print(game_file, output_dir)


if __name__ == "__main__":
	main()
