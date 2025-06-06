#!/usr/bin/env python3
"""
🧪 Simple Redis/DragonflyDB Connection Test

This Python script tests the connection to DragonflyDB Cloud
to verify if the issue is with Rust/TLS or with the connection itself.
"""

import redis
import os
import sys
from dotenv import load_dotenv

def test_connection():
    print("🧪 === SIMPLE DRAGONFLYDB CLOUD CONNECTION TEST ===")
    
    # Load environment variables
    load_dotenv()
    
    dragonfly_url = os.getenv('DRAGONFLY_URL')
    if not dragonfly_url:
        print("❌ DRAGONFLY_URL not found in .env file")
        return False
    
    print(f"🔗 Connection URL: {dragonfly_url}")
    
    try:
        # Test connection with Python redis client
        print("📡 Attempting connection...")
        
        r = redis.from_url(dragonfly_url, decode_responses=True)
        
        # Test PING
        print("🏓 Sending PING...")
        result = r.ping()
        
        if result:
            print("🎉 === CONNECTION SUCCESSFUL! ===")
            print("✅ PING returned True")
            
            # Test basic operations
            print("🧪 Testing basic operations...")
            
            # SET test
            test_key = "sniperbot_python_test"
            test_value = "connection_works"
            
            r.set(test_key, test_value)
            print("✅ SET operation successful")
            
            # GET test
            retrieved = r.get(test_key)
            if retrieved == test_value:
                print("✅ GET operation successful")
                
                # Cleanup
                r.delete(test_key)
                print("✅ Cleanup completed")
                
                # Check if we have any SniperBot data
                print("📊 Checking for SniperBot data...")
                
                raw_opportunities = r.llen('all_raw_opportunities')
                processed_tokens = r.scard('processed_tokens')
                
                print(f"  • Raw opportunities: {raw_opportunities}")
                print(f"  • Processed tokens: {processed_tokens}")
                
                if raw_opportunities > 0:
                    print("🎯 SniperBot data is available!")
                    print("🚀 Ready for autonomous operation!")
                else:
                    print("📝 No SniperBot data yet")
                    print("💡 Run Soul Meteor Scanner to populate data")
                
                return True
            else:
                print(f"❌ GET returned wrong value: {retrieved}")
                return False
        else:
            print("❌ PING returned False")
            return False
            
    except redis.ConnectionError as e:
        print(f"❌ Connection Error: {e}")
        print("🔧 Possible issues:")
        print("   • Network connectivity")
        print("   • Firewall blocking port 6385")
        print("   • Invalid credentials")
        print("   • DragonflyDB Cloud service down")
        return False
        
    except redis.AuthenticationError as e:
        print(f"❌ Authentication Error: {e}")
        print("🔧 Check credentials in DRAGONFLY_URL")
        return False
        
    except Exception as e:
        print(f"❌ Unexpected Error: {e}")
        print(f"   Error type: {type(e).__name__}")
        return False

if __name__ == "__main__":
    success = test_connection()
    
    if success:
        print("\n🎉 === PYTHON CONNECTION TEST PASSED ===")
        print("✅ DragonflyDB Cloud is accessible from Python")
        print("🔧 Issue is likely with Rust TLS configuration")
        print("")
        print("🎯 NEXT STEPS:")
        print("   1. Fix Rust TLS configuration")
        print("   2. Test Rust connection")
        print("   3. Run autonomous bot")
        sys.exit(0)
    else:
        print("\n❌ === PYTHON CONNECTION TEST FAILED ===")
        print("🔧 Fix connection issues before proceeding")
        sys.exit(1)
