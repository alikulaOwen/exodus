from typing import List

class Transaction:
    def __init__(self, tx_id: str, amount: float, description: str):
        self.tx_id: str = tx_id
        self.amount: float = amount
        self.description: str = description

class BankAccount:
    def __init__(self, owner: str, initial_balance: float):
        self.owner: str = owner
        self.balance: float = initial_balance
        self.transactions: List[Transaction] = []

    def deposit(self, amount: float, description: str) -> bool:
        if amount <= 0.0:
            return False
        self.balance += amount
        tx = Transaction("tx_dep", amount, description)
        self.transactions.append(tx)
        return True

    def withdraw(self, amount: float, description: str) -> bool:
        if amount <= 0.0 or amount > self.balance:
            return False
        self.balance -= amount
        tx = Transaction("tx_wth", amount, description)
        self.transactions.append(tx)
        return True

    def get_balance(self) -> float:
        return self.balance
