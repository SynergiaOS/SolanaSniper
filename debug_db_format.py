#!/usr/bin/env python3
"""
🔍 Debug Database Format

This script inspects the format of data in DragonflyDB to understand
why Rust deserialization is failing.
"""

import redis
import os
import json
from dotenv import load_dotenv

def debug_db_format():
    print("🔍 === DEBUGGING DRAGONFLYDB DATA FORMAT ===")
    
    # Load environment variables
    load_dotenv()
    
    dragonfly_url = os.getenv('DRAGONFLY_URL')
    if not dragonfly_url:
        print("❌ DRAGONFLY_URL not found in .env file")
        return False
    
    print(f"🔗 Connection URL: {dragonfly_url}")
    
    try:
        r = redis.from_url(dragonfly_url, decode_responses=True)
        
        # Test connection
        r.ping()
        print("✅ Connected to DragonflyDB")
        
        # Check what keys exist
        print("\n📋 === AVAILABLE KEYS ===")
        all_keys = r.keys("*")
        print(f"Total keys: {len(all_keys)}")
        
        for key in sorted(all_keys)[:20]:  # Show first 20 keys
            key_type = r.type(key)
            print(f"  • {key} ({key_type})")
        
        if len(all_keys) > 20:
            print(f"  ... and {len(all_keys) - 20} more keys")
        
        # Check specific key that Rust is trying to read
        target_key = "all_raw_opportunities"
        print(f"\n🎯 === INSPECTING KEY: {target_key} ===")
        
        if r.exists(target_key):
            key_type = r.type(target_key)
            print(f"Key type: {key_type}")
            
            if key_type == "list":
                length = r.llen(target_key)
                print(f"List length: {length}")
                
                if length > 0:
                    # Get first few items
                    items = r.lrange(target_key, 0, 4)  # First 5 items
                    print(f"First {min(5, length)} items:")
                    
                    for i, item in enumerate(items):
                        print(f"\n  Item {i+1}:")
                        print(f"    Type: {type(item)}")
                        print(f"    Length: {len(item) if isinstance(item, str) else 'N/A'}")
                        print(f"    Content preview: {item[:200]}...")
                        
                        # Try to parse as JSON
                        try:
                            parsed = json.loads(item)
                            print(f"    ✅ Valid JSON")
                            print(f"    JSON keys: {list(parsed.keys()) if isinstance(parsed, dict) else 'Not a dict'}")
                        except json.JSONDecodeError as e:
                            print(f"    ❌ Invalid JSON: {e}")
                            print(f"    Raw content: {repr(item[:100])}")
                
            elif key_type == "string":
                value = r.get(target_key)
                print(f"String value: {value}")
                
            elif key_type == "set":
                members = r.smembers(target_key)
                print(f"Set members: {list(members)[:10]}")  # First 10 members
                
            elif key_type == "hash":
                fields = r.hgetall(target_key)
                print(f"Hash fields: {list(fields.keys())[:10]}")  # First 10 fields
                
        else:
            print(f"❌ Key '{target_key}' does not exist")
            
            # Look for similar keys
            similar_keys = [k for k in all_keys if "opportunity" in k.lower() or "raw" in k.lower()]
            if similar_keys:
                print(f"\n🔍 Similar keys found:")
                for key in similar_keys[:10]:
                    print(f"  • {key}")
        
        # Check processed tokens
        processed_key = "processed_tokens"
        print(f"\n🎯 === INSPECTING KEY: {processed_key} ===")
        
        if r.exists(processed_key):
            key_type = r.type(processed_key)
            print(f"Key type: {key_type}")
            
            if key_type == "set":
                count = r.scard(processed_key)
                print(f"Set size: {count}")
                
                if count > 0:
                    members = list(r.smembers(processed_key))[:5]
                    print(f"Sample members: {members}")
        else:
            print(f"❌ Key '{processed_key}' does not exist")
        
        # Check for any raw opportunity keys
        print(f"\n🔍 === SEARCHING FOR RAW OPPORTUNITY KEYS ===")
        raw_opp_keys = [k for k in all_keys if k.startswith("raw_opportunity:")]
        print(f"Found {len(raw_opp_keys)} raw opportunity keys")
        
        if raw_opp_keys:
            sample_key = raw_opp_keys[0]
            print(f"\n📋 Sample raw opportunity key: {sample_key}")
            
            key_type = r.type(sample_key)
            print(f"Type: {key_type}")
            
            if key_type == "string":
                value = r.get(sample_key)
                print(f"Value preview: {value[:300]}...")
                
                try:
                    parsed = json.loads(value)
                    print(f"✅ Valid JSON")
                    print(f"JSON structure: {json.dumps(parsed, indent=2)[:500]}...")
                except json.JSONDecodeError as e:
                    print(f"❌ Invalid JSON: {e}")
        
        return True
        
    except Exception as e:
        print(f"❌ Error: {e}")
        return False

if __name__ == "__main__":
    debug_db_format()
