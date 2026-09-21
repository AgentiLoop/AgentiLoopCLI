#!/usr/bin/env python3
"""
Tic Tac Toe - Human vs Computer
Play by entering numbers 1-9 corresponding to board positions
"""

import random
import sys


class TicTacToe:
    def __init__(self):
        self.board = [' ' for _ in range(9)]
        self.human = 'X'
        self.computer = 'O'
    
    def print_board(self):
        """Display the current board state"""
        print("\n")
        print(f" {self.board[0]} | {self.board[1]} | {self.board[2]} ")
        print("---+---+---")
        print(f" {self.board[3]} | {self.board[4]} | {self.board[5]} ")
        print("---+---+---")
        print(f" {self.board[6]} | {self.board[7]} | {self.board[8]} ")
        print("\n")
    
    def print_positions(self):
        """Show position numbers for reference"""
        print("\nPosition numbers:")
        print(" 1 | 2 | 3 ")
        print("---+---+---")
        print(" 4 | 5 | 6 ")
        print("---+---+---")
        print(" 7 | 8 | 9 ")
        print()
    
    def available_moves(self):
        """Return list of available positions"""
        return [i for i, spot in enumerate(self.board) if spot == ' ']
    
    def make_move(self, position, player):
        """Place player's mark at position"""
        self.board[position] = player
    
    def check_winner(self, player):
        """Check if player has won"""
        win_conditions = [
            [0, 1, 2], [3, 4, 5], [6, 7, 8],  # rows
            [0, 3, 6], [1, 4, 7], [2, 5, 8],  # columns
            [0, 4, 8], [2, 4, 6]              # diagonals
        ]
        return any(all(self.board[i] == player for i in combo) for combo in win_conditions)
    
    def is_board_full(self):
        """Check if board is full (tie game)"""
        return ' ' not in self.board
    
    def get_human_move(self):
        """Get valid move from human player"""
        while True:
            try:
                move = input("Enter your move (1-9): ").strip()
                if not move:
                    continue
                
                position = int(move) - 1
                
                if position < 0 or position > 8:
                    print("Please enter a number between 1 and 9")
                    continue
                
                if position not in self.available_moves():
                    print("That position is already taken!")
                    continue
                
                return position
            
            except ValueError:
                print("Invalid input! Please enter a number between 1 and 9")
            except KeyboardInterrupt:
                print("\n\nGame cancelled. Goodbye!")
                sys.exit(0)
    
    def get_computer_move(self):
        """AI logic for computer move"""
        available = self.available_moves()
        
        # Try to win
        for move in available:
            self.board[move] = self.computer
            if self.check_winner(self.computer):
                self.board[move] = ' '
                return move
            self.board[move] = ' '
        
        # Block human from winning
        for move in available:
            self.board[move] = self.human
            if self.check_winner(self.human):
                self.board[move] = ' '
                return move
            self.board[move] = ' '
        
        # Take center if available
        if 4 in available:
            return 4
        
        # Take a corner
        corners = [i for i in [0, 2, 6, 8] if i in available]
        if corners:
            return random.choice(corners)
        
        # Take any remaining spot
        return random.choice(available)
    
    def play(self):
        """Main game loop"""
        print("=" * 40)
        print("  Welcome to Tic Tac Toe!")
        print("=" * 40)
        print(f"\nYou are '{self.human}' and the computer is '{self.computer}'")
        self.print_positions()
        
        while True:
            # Human's turn
            self.print_board()
            print("Your turn!")
            human_move = self.get_human_move()
            self.make_move(human_move, self.human)
            
            if self.check_winner(self.human):
                self.print_board()
                print("🎉 Congratulations! You win! 🎉")
                break
            
            if self.is_board_full():
                self.print_board()
                print("😐 It's a tie! 😐")
                break
            
            # Computer's turn
            print("\nComputer is thinking...")
            computer_move = self.get_computer_move()
            self.make_move(computer_move, self.computer)
            print(f"Computer chose position {computer_move + 1}")
            
            if self.check_winner(self.computer):
                self.print_board()
                print("💻 Computer wins! Better luck next time! 💻")
                break
            
            if self.is_board_full():
                self.print_board()
                print("😐 It's a tie! 😐")
                break
        
        # Ask to play again
        while True:
            again = input("\nPlay again? (y/n): ").strip().lower()
            if again in ['y', 'yes']:
                self.__init__()
                self.play()
                break
            elif again in ['n', 'no']:
                print("\nThanks for playing! Goodbye! 👋")
                break
            else:
                print("Please enter 'y' or 'n'")


if __name__ == "__main__":
    game = TicTacToe()
    game.play()
