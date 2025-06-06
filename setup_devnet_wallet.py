#!/usr/bin/env python3
"""
SniperBot 2.0 - Devnet Wallet Setup
Creates a new wallet for devnet testing and requests SOL from faucet
"""

import json
import os
import requests
import time
from solders.keypair import Keypair
from solders.pubkey import Pubkey
from solana.rpc.api import Client
from solana.rpc.types import TxOpts
from solana.rpc.commitment import Confirmed

def create_devnet_wallet():
    """Create a new wallet for devnet"""
    print("🔧 Creating new devnet wallet...")
    
    # Generate new keypair
    keypair = Keypair()
    public_key = str(keypair.pubkey())
    private_key = list(keypair.secret())
    
    print(f"✅ Wallet created!")
    print(f"📍 Public Key: {public_key}")
    print(f"🔐 Private Key: {private_key}")
    
    # Save to file
    wallet_data = {
        "public_key": public_key,
        "private_key": private_key,
        "network": "devnet",
        "created_at": time.time()
    }
    
    os.makedirs(".keys", exist_ok=True)
    with open(".keys/devnet_wallet.json", "w") as f:
        json.dump(wallet_data, f, indent=2)
    
    print("💾 Wallet saved to .keys/devnet_wallet.json")
    return public_key, private_key

def request_devnet_sol(public_key):
    """Request SOL from Solana devnet faucet"""
    print(f"💰 Requesting devnet SOL for {public_key}...")
    
    # Solana devnet faucet endpoint
    faucet_url = "https://api.devnet.solana.com"
    
    try:
        # Connect to devnet
        client = Client(faucet_url)
        
        # Request airdrop (2 SOL = 2,000,000,000 lamports)
        response = client.request_airdrop(
            Pubkey.from_string(public_key), 
            2_000_000_000,  # 2 SOL
            commitment=Confirmed
        )
        
        if response.value:
            print(f"✅ Airdrop requested! Transaction: {response.value}")
            print("⏳ Waiting for confirmation...")
            
            # Wait for confirmation
            time.sleep(15)
            
            # Check balance
            balance_response = client.get_balance(Pubkey.from_string(public_key))
            if balance_response.value:
                balance_sol = balance_response.value / 1_000_000_000
                print(f"💰 Current balance: {balance_sol} SOL")
                return True
            else:
                print("❌ Could not check balance")
                return False
        else:
            print("❌ Airdrop request failed")
            return False
            
    except Exception as e:
        print(f"❌ Error requesting airdrop: {e}")
        print("💡 You can manually request SOL from: https://faucet.solana.com/")
        return False

def update_env_for_devnet(public_key, private_key):
    """Update .env file with devnet configuration"""
    print("🔧 Updating .env for devnet...")
    
    # Read current .env
    env_lines = []
    if os.path.exists(".env"):
        with open(".env", "r") as f:
            env_lines = f.readlines()
    
    # Update relevant lines
    updated_lines = []
    for line in env_lines:
        if line.startswith("SOLANA_NETWORK="):
            updated_lines.append("SOLANA_NETWORK=devnet\n")
        elif line.startswith("SOLANA_PUBLIC_KEY="):
            updated_lines.append(f'SOLANA_PUBLIC_KEY="{public_key}"\n')
        elif line.startswith("SOLANA_PRIVATE_KEY="):
            # Convert private key list to base58 string for compatibility
            private_key_str = ",".join(map(str, private_key))
            updated_lines.append(f'SOLANA_PRIVATE_KEY="[{private_key_str}]"\n')
        elif line.startswith("SOLANA_WALLET_PATH="):
            updated_lines.append('SOLANA_WALLET_PATH=".keys/devnet_wallet.json"\n')
        elif line.startswith("DRY_RUN="):
            updated_lines.append("DRY_RUN=false\n")  # Enable real transactions on devnet
        else:
            updated_lines.append(line)
    
    # Write updated .env
    with open(".env", "w") as f:
        f.writelines(updated_lines)
    
    print("✅ .env updated for devnet")

def main():
    print("🌟 SniperBot 2.0 - Devnet Setup")
    print("=" * 50)
    
    # Create wallet
    public_key, private_key = create_devnet_wallet()
    
    # Request SOL from faucet
    success = request_devnet_sol(public_key)
    
    if not success:
        print("\n💡 Manual faucet instructions:")
        print("1. Go to https://faucet.solana.com/")
        print(f"2. Enter your public key: {public_key}")
        print("3. Select 'Devnet' network")
        print("4. Request 1-2 SOL for testing")
    
    # Update .env for devnet
    update_env_for_devnet(public_key, private_key)
    
    print("\n🎯 Devnet Setup Complete!")
    print("=" * 50)
    print(f"📍 Public Key: {public_key}")
    print("💾 Wallet saved to: .keys/devnet_wallet.json")
    print("🔧 .env updated for devnet")
    print("🌐 Network: Helius Devnet (Enhanced APIs)")
    print("\n🚀 Ready for devnet testing!")
    print("Run: cargo run --bin hybrid_system")

if __name__ == "__main__":
    main()
