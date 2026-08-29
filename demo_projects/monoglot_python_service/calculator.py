def calculate_fee(amount: float, rate: float) -> float:
    return amount * rate

def format_currency(amount: float) -> str:
    return f"${amount:.2f}"
