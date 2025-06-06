#!/usr/bin/env python3
"""
🔍 Check Wallet Address from Private Key

This script derives the public address from your private key and updates the .env file.
"""

import os
import json
from solders.keypair import Keypair
import base58
from dotenv import load_dotenv

def check_wallet_from_private_key():
    print("🔍 === CHECKING WALLET ADDRESS FROM PRIVATE KEY ===")
    
    # Load environment variables
    load_dotenv()
    
    private_key_b58 = os.getenv('SOLANA_PRIVATE_KEY')
    if not private_key_b58:
        print("❌ SOLANA_PRIVATE_KEY not found in .env file")
        return None
    
    print(f"🔑 Private Key: {private_key_b58[:8]}...{private_key_b58[-8:]}")
    
    try:
        # Decode private key from Base58
        private_key_bytes = base58.b58decode(private_key_b58)
        
        # Create keypair from private key
        keypair = Keypair.from_bytes(private_key_bytes)
        
        # Get public key (wallet address)
        public_key = keypair.pubkey()
        
        print(f"✅ Wallet Address: {public_key}")
        
        # Create wallet file for Solana CLI compatibility
        wallet_data = list(private_key_bytes)
        
        # Create .keys directory if it doesn't exist
        os.makedirs('.keys', exist_ok=True)
        
        # Save wallet file
        wallet_file = '.keys/sniperbot_wallet.json'
        with open(wallet_file, 'w') as f:
            json.dump(wallet_data, f)
        
        print(f"💾 Wallet file saved: {wallet_file}")
        
        # Update .env file with correct public key
        env_content = ""
        with open('.env', 'r') as f:
            env_content = f.read()
        
        # Replace the public key line
        updated_content = env_content.replace(
            'SOLANA_PUBLIC_KEY="TBD"',
            f'SOLANA_PUBLIC_KEY="{public_key}"'
        )
        
        with open('.env', 'w') as f:
            f.write(updated_content)
        
        print(f"✅ Updated .env with public key: {public_key}")
        
        return {
            'public_key': str(public_key),
            'private_key': private_key_b58,
            'wallet_file': wallet_file
        }
        
    except Exception as e:
        print(f"❌ Error processing private key: {e}")
        return None

def check_wallet_balance(public_key):
    """Check wallet balance using Helius RPC"""
    import requests
    
    helius_rpc = os.getenv('HELIUS_RPC_URL')
    if not helius_rpc:
        print("❌ HELIUS_RPC_URL not found in .env")
        return
    
    print(f"\n💰 === CHECKING WALLET BALANCE ===")
    print(f"🔗 RPC: {helius_rpc}")
    print(f"📍 Address: {public_key}")
    
    try:
        balance_request = {
            "jsonrpc": "2.0",
            "id": 1,
            "method": "getBalance",
            "params": [str(public_key)]
        }
        
        response = requests.post(helius_rpc, json=balance_request)
        
        if response.status_code == 200:
            result = response.json()
            
            if 'result' in result and 'value' in result['result']:
                lamports = result['result']['value']
                sol_balance = lamports / 1_000_000_000.0
                
                print(f"💰 Balance: {sol_balance} SOL ({lamports:,} lamports)")
                
                if sol_balance > 0:
                    print("✅ Wallet has funds - ready for trading!")
                    
                    # Trading recommendations based on balance
                    if sol_balance < 0.1:
                        print("💡 Balance < 0.1 SOL - Good for DRY RUN testing")
                    elif sol_balance < 0.5:
                        print("💡 Balance < 0.5 SOL - Good for PumpFun sniping")
                    elif sol_balance < 2.0:
                        print("💡 Balance < 2.0 SOL - Good for PumpFun + Liquidity sniping")
                    else:
                        print("💡 Balance > 2.0 SOL - Ready for all strategies!")
                        
                    # Calculate USD value (approximate)
                    sol_price_usd = 200  # Approximate SOL price
                    usd_value = sol_balance * sol_price_usd
                    print(f"💵 Approximate USD value: ${usd_value:.2f}")
                    
                else:
                    print("❌ Wallet is empty!")
                    print(f"💰 Send SOL to: {public_key}")
                    
            else:
                print("❌ Unexpected response format")
                print(f"Response: {result}")
                
        else:
            print(f"❌ RPC request failed: {response.status_code}")
            
    except Exception as e:
        print(f"❌ Balance check error: {e}")

def main():
    print("🤖 === SNIPERBOT WALLET ADDRESS CHECK ===")
    
    wallet_info = check_wallet_from_private_key()
    
    if wallet_info:
        print("\n✅ === WALLET SETUP COMPLETE ===")
        print(f"📍 Address: {wallet_info['public_key']}")
        print(f"💾 Wallet file: {wallet_info['wallet_file']}")
        
        # Check balance
        check_wallet_balance(wallet_info['public_key'])
        
        print("\n🚀 === NEXT STEPS ===")
        print("1. ✅ Wallet configured successfully")
        print("2. 💰 Fund wallet if balance is low")
        print("3. 🧪 Test with autonomous bot")
        print("4. 📊 Monitor dashboard")
        
    else:
        print("\n❌ Wallet setup failed")

if __name__ == "__main__":
    main()
