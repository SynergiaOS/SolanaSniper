#!/usr/bin/env python3
"""
🔐 SniperBot Wallet Setup

This script helps you securely set up a wallet for SniperBot trading.
ALWAYS use a dedicated wallet for bot trading, never your main wallet!
"""

import os
import json
from solders.keypair import Keypair
from solders.pubkey import Pubkey
import base58

def create_new_wallet():
    """Create a new Solana wallet for bot trading"""
    print("🔐 === CREATING NEW SOLANA WALLET FOR SNIPERBOT ===")
    print("⚠️  IMPORTANT: This will be a DEDICATED wallet for bot trading only!")
    print("⚠️  Never use your main wallet for automated trading!")
    print()
    
    # Generate new keypair
    keypair = Keypair()
    
    # Get public key (wallet address)
    public_key = keypair.pubkey()
    
    # Get private key in different formats
    private_key_bytes = bytes(keypair)
    private_key_base58 = base58.b58encode(private_key_bytes).decode('utf-8')
    private_key_array = list(private_key_bytes)
    
    print(f"✅ New wallet created successfully!")
    print(f"📍 Public Address: {public_key}")
    print(f"🔑 Private Key (Base58): {private_key_base58}")
    print()
    
    # Create wallet file for Solana CLI
    wallet_data = private_key_array
    
    # Create .keys directory if it doesn't exist
    os.makedirs('.keys', exist_ok=True)
    
    # Save wallet file
    wallet_file = '.keys/sniperbot_wallet.json'
    with open(wallet_file, 'w') as f:
        json.dump(wallet_data, f)
    
    print(f"💾 Wallet saved to: {wallet_file}")
    print()
    
    # Create .env entry
    env_entry = f"""
# SniperBot Wallet Configuration
SOLANA_PRIVATE_KEY="{private_key_base58}"
SOLANA_WALLET_PATH=".keys/sniperbot_wallet.json"
SOLANA_PUBLIC_KEY="{public_key}"
"""
    
    print("📝 Add this to your .env file:")
    print("=" * 50)
    print(env_entry)
    print("=" * 50)
    print()
    
    # Security instructions
    print("🔐 === SECURITY INSTRUCTIONS ===")
    print("1. ✅ BACKUP your private key safely (write it down offline)")
    print("2. ✅ NEVER share your private key with anyone")
    print("3. ✅ Add .keys/ to .gitignore (already done)")
    print("4. ✅ Start with small amounts for testing")
    print("5. ✅ Monitor bot activity closely")
    print()
    
    # Funding instructions
    print("💰 === FUNDING INSTRUCTIONS ===")
    print(f"1. Send SOL to this address: {public_key}")
    print("2. Recommended starting amount: 0.1-0.2 SOL (~$20-40)")
    print("3. Keep rest of your $80 for gradual increases")
    print("4. Always test with small amounts first!")
    print()
    
    # Next steps
    print("🚀 === NEXT STEPS ===")
    print("1. Fund the wallet with small amount (0.1 SOL)")
    print("2. Update .env file with wallet configuration")
    print("3. Test wallet connection with SniperBot")
    print("4. Start with DRY RUN mode")
    print("5. Gradually increase amounts as you gain confidence")
    print()
    
    return {
        'public_key': str(public_key),
        'private_key_base58': private_key_base58,
        'wallet_file': wallet_file
    }

def check_existing_wallet():
    """Check if wallet already exists"""
    wallet_file = '.keys/sniperbot_wallet.json'
    
    if os.path.exists(wallet_file):
        print(f"✅ Found existing wallet: {wallet_file}")
        
        try:
            with open(wallet_file, 'r') as f:
                wallet_data = json.load(f)
            
            # Recreate keypair from saved data
            keypair = Keypair.from_bytes(bytes(wallet_data))
            public_key = keypair.pubkey()
            
            print(f"📍 Wallet Address: {public_key}")
            print("💡 Use this address to fund your bot wallet")
            
            return {
                'public_key': str(public_key),
                'wallet_file': wallet_file,
                'exists': True
            }
            
        except Exception as e:
            print(f"❌ Error reading wallet file: {e}")
            return None
    
    return None

def main():
    print("🤖 === SNIPERBOT WALLET SETUP ===")
    print()
    
    # Check if wallet already exists
    existing = check_existing_wallet()
    
    if existing and existing.get('exists'):
        print("✅ Wallet already configured!")
        choice = input("Do you want to create a NEW wallet? (y/N): ").lower()
        if choice != 'y':
            print("Using existing wallet.")
            return existing
    
    # Create new wallet
    wallet_info = create_new_wallet()
    
    print("🎯 === WALLET SETUP COMPLETE ===")
    print("Your SniperBot wallet is ready for trading!")
    
    return wallet_info

if __name__ == "__main__":
    try:
        wallet_info = main()
        print("\n✅ Wallet setup completed successfully!")
        
    except KeyboardInterrupt:
        print("\n❌ Setup cancelled by user")
    except Exception as e:
        print(f"\n❌ Setup failed: {e}")
        print("Please check your Python environment and try again")
