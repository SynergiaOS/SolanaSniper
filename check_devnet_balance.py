#!/usr/bin/env python3
"""
Check devnet wallet balance
"""

import json
from solders.pubkey import Pubkey
from solana.rpc.api import Client

def check_balance():
    # Load wallet
    with open(".keys/devnet_wallet.json", "r") as f:
        wallet = json.load(f)
    
    public_key = wallet["public_key"]
    print(f"🔍 Checking balance for: {public_key}")
    
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
                return True
            else:
                print("❌ Wallet not funded. Please use faucet:")
                print(f"   https://faucet.solana.com/")
                print(f"   Address: {public_key}")
                print(f"   Network: Devnet")
                return False
        else:
            print("❌ Could not retrieve balance")
            return False
            
    except Exception as e:
        print(f"❌ Error: {e}")
        return False

if __name__ == "__main__":
    check_balance()
