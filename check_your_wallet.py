#!/usr/bin/env python3
"""
Check your actual devnet wallet balance
"""

from solders.pubkey import Pubkey
from solana.rpc.api import Client

def check_balance():
    # Your actual wallet address
    public_key = "HhCMHCECoKmSwiQHFQ7mKJR5ahCDMZrEyoS9eZWgnXeh"
    print(f"🔍 Checking balance for YOUR wallet: {public_key}")
    
    # Connect to devnet
    client = Client("https://api.devnet.solana.com")
    
    try:
        # Get balance
        response = client.get_balance(Pubkey.from_string(public_key))
        if response.value is not None:
            balance_lamports = response.value
            balance_sol = balance_lamports / 1_000_000_000
            print(f"💰 Balance: {balance_sol} SOL ({balance_lamports} lamports)")
            
            if balance_sol > 0:
                print("✅ Wallet funded! Ready for devnet testing!")
                print(f"🚀 We can run Hybrid System with {balance_sol} SOL!")
                return True
            else:
                print("❌ Wallet not funded.")
                return False
        else:
            print("❌ Could not retrieve balance")
            return False
            
    except Exception as e:
        print(f"❌ Error: {e}")
        return False

if __name__ == "__main__":
    check_balance()
