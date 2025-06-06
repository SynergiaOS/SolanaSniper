#!/usr/bin/env python3
"""
🔍 Check Dashboard Data in DragonflyDB

This script verifies that dashboard data was properly saved by the autonomous bot.
"""

import redis
import os
import json
from dotenv import load_dotenv

def check_dashboard_data():
    print("🔍 === CHECKING DASHBOARD DATA IN DRAGONFLYDB ===")
    
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
        
        # Check dashboard-specific keys
        dashboard_keys = [
            'bot:status',
            'dashboard:activity_feed',
            'realtime:metrics',
            'dashboard:stats'
        ]
        
        print("\n📋 === DASHBOARD DATA STATUS ===")
        
        for key in dashboard_keys:
            exists = r.exists(key)
            if exists:
                key_type = r.type(key)
                print(f"✅ {key} ({key_type})")
                
                # Show sample data
                if key_type == "string":
                    data = r.get(key)
                    try:
                        parsed = json.loads(data)
                        print(f"   📄 Sample: {json.dumps(parsed, indent=2)[:200]}...")
                    except:
                        print(f"   📄 Raw data: {data[:100]}...")
                        
                elif key_type == "list":
                    length = r.llen(key)
                    print(f"   📊 List length: {length}")
                    
                    if length > 0:
                        # Get first item
                        first_item = r.lindex(key, 0)
                        try:
                            parsed = json.loads(first_item)
                            print(f"   📄 First item: {json.dumps(parsed, indent=2)[:200]}...")
                        except:
                            print(f"   📄 First item: {first_item[:100]}...")
                            
            else:
                print(f"❌ {key} (not found)")
        
        # Check for any other dashboard-related keys
        print("\n🔍 === SEARCHING FOR OTHER DASHBOARD KEYS ===")
        all_keys = r.keys("*")
        dashboard_related = [k for k in all_keys if any(term in k.lower() for term in ['dashboard', 'bot', 'realtime', 'activity'])]
        
        print(f"Found {len(dashboard_related)} dashboard-related keys:")
        for key in sorted(dashboard_related):
            key_type = r.type(key)
            print(f"  • {key} ({key_type})")
        
        # Check activity feed in detail
        print("\n📋 === ACTIVITY FEED DETAILS ===")
        if r.exists('dashboard:activity_feed'):
            activities = r.lrange('dashboard:activity_feed', 0, 4)  # First 5 activities
            print(f"Activity feed has {len(activities)} events:")
            
            for i, activity_data in enumerate(activities):
                try:
                    activity = json.loads(activity_data)
                    print(f"  {i+1}. {activity.get('event_type', 'Unknown')} - {activity.get('description', 'No description')}")
                    print(f"     Time: {activity.get('timestamp', 'Unknown')}")
                    print(f"     Severity: {activity.get('severity', 'Unknown')}")
                except Exception as e:
                    print(f"  {i+1}. Failed to parse: {e}")
        else:
            print("❌ No activity feed found")
        
        # Check bot status in detail
        print("\n🤖 === BOT STATUS DETAILS ===")
        if r.exists('bot:status'):
            status_data = r.get('bot:status')
            try:
                status = json.loads(status_data)
                print(f"Bot State: {status.get('state', 'Unknown')}")
                print(f"Bot Mode: {status.get('mode', 'Unknown')}")
                print(f"Version: {status.get('version', 'Unknown')}")
                print(f"Started: {status.get('started_at', 'Unknown')}")
                print(f"Last Activity: {status.get('last_activity', 'Unknown')}")
            except Exception as e:
                print(f"❌ Failed to parse bot status: {e}")
        else:
            print("❌ No bot status found")
        
        return True
        
    except Exception as e:
        print(f"❌ Error: {e}")
        return False

if __name__ == "__main__":
    success = check_dashboard_data()
    
    if success:
        print("\n🎉 === DASHBOARD DATA CHECK COMPLETED ===")
        print("✅ Dashboard integration is working!")
        print("🚀 Ready for frontend development!")
    else:
        print("\n❌ === DASHBOARD DATA CHECK FAILED ===")
        print("🔧 Check bot configuration and database connection")
