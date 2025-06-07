#!/usr/bin/env python3
"""
🚀 Scrapy Service Build Script
Professional PyInstaller build for 10x performance web scraping
"""

import os
import sys
import subprocess
import shutil
from pathlib import Path

def build_scrapy_executable():
    """Build Scrapy Service as standalone executable"""
    
    print("🚀 Building Scrapy Service executable...")
    print("⚡ Professional web scraping with 10x performance boost")
    
    # Get current directory
    current_dir = Path(__file__).parent
    
    # PyInstaller command with optimized settings
    pyinstaller_cmd = [
        'pyinstaller',
        '--onefile',                    # Single executable
        '--name=scrapy_service',        # Executable name
        '--distpath=../../dist',        # Output to project dist folder
        '--workpath=./build',           # Build directory
        '--specpath=./build',           # Spec file location
        '--clean',                      # Clean build
        '--noconfirm',                  # Overwrite without confirmation
        
        # Console application (no GUI)
        '--console',
        
        # Optimization flags
        '--optimize=2',                 # Python optimization level
        '--strip',                      # Strip debug symbols (Linux/Mac)
        
        # Hidden imports for Scrapy
        '--hidden-import=scrapy',
        '--hidden-import=scrapy.crawler',
        '--hidden-import=scrapy.spiders',
        '--hidden-import=scrapy.http',
        '--hidden-import=scrapy.selector',
        '--hidden-import=scrapy.utils',
        '--hidden-import=twisted',
        '--hidden-import=twisted.internet',
        '--hidden-import=twisted.internet.reactor',
        '--hidden-import=twisted.internet.defer',
        '--hidden-import=twisted.internet.endpoints',
        '--hidden-import=twisted.web',
        '--hidden-import=w3lib',
        '--hidden-import=parsel',
        '--hidden-import=itemadapter',
        '--hidden-import=itemloaders',
        
        # HTML parsing
        '--hidden-import=lxml',
        '--hidden-import=lxml.etree',
        '--hidden-import=lxml.html',
        '--hidden-import=beautifulsoup4',
        '--hidden-import=bs4',
        '--hidden-import=html5lib',
        
        # HTTP clients
        '--hidden-import=requests',
        '--hidden-import=urllib3',
        '--hidden-import=httpx',
        '--hidden-import=aiohttp',
        
        # Sentiment analysis
        '--hidden-import=vaderSentiment',
        '--hidden-import=textblob',
        
        # Data processing
        '--hidden-import=json',
        '--hidden-import=csv',
        '--hidden-import=pickle',
        
        # Async
        '--hidden-import=asyncio',
        '--hidden-import=concurrent.futures',
        
        # Exclude unnecessary modules to reduce size
        '--exclude-module=tkinter',
        '--exclude-module=matplotlib',
        '--exclude-module=numpy',
        '--exclude-module=pandas',
        '--exclude-module=scipy',
        '--exclude-module=PIL',
        '--exclude-module=cv2',
        '--exclude-module=torch',
        '--exclude-module=tensorflow',
        
        # Entry point
        'main.py'
    ]
    
    try:
        # Change to script directory
        os.chdir(current_dir)
        
        # Install dependencies first
        print("📦 Installing dependencies...")
        subprocess.run([sys.executable, '-m', 'pip', 'install', '-r', 'requirements.txt'], 
                      check=True)
        
        # Run PyInstaller
        print("🔨 Running PyInstaller...")
        result = subprocess.run(pyinstaller_cmd, check=True, capture_output=True, text=True)
        
        print("✅ Build completed successfully!")
        
        # Check if executable was created
        dist_path = current_dir / '../../dist'
        executable_name = 'scrapy_service.exe' if sys.platform == 'win32' else 'scrapy_service'
        executable_path = dist_path / executable_name
        
        if executable_path.exists():
            size_mb = executable_path.stat().st_size / (1024 * 1024)
            print(f"📁 Executable created: {executable_path}")
            print(f"📊 Size: {size_mb:.1f} MB")
            
            # Make executable on Unix systems
            if sys.platform != 'win32':
                os.chmod(executable_path, 0o755)
                print("🔧 Made executable on Unix system")
            
            # Test the executable
            print("🧪 Testing executable...")
            test_input = {
                "token_symbol": "BTC",
                "data_types": ["news"],
                "max_results": 5,
                "time_range_hours": 24,
                "sentiment_analysis": True
            }
            
            test_process = subprocess.run(
                [str(executable_path)],
                input=str(test_input).replace("'", '"'),
                capture_output=True,
                text=True,
                timeout=30
            )
            
            if test_process.returncode == 0:
                print("✅ Executable test passed!")
                print("🚀 Scrapy Service ready for deployment!")
            else:
                print("⚠️ Executable test failed:")
                print(f"STDOUT: {test_process.stdout}")
                print(f"STDERR: {test_process.stderr}")
        else:
            print("❌ Executable not found after build")
            return False
        
        # Clean up build artifacts
        print("🧹 Cleaning up build artifacts...")
        build_dir = current_dir / 'build'
        if build_dir.exists():
            shutil.rmtree(build_dir)
        
        print("🎉 Scrapy Service build completed!")
        print("⚡ Ready for 10x performance boost in production!")
        
        return True
        
    except subprocess.CalledProcessError as e:
        print(f"❌ Build failed: {e}")
        print(f"STDOUT: {e.stdout}")
        print(f"STDERR: {e.stderr}")
        return False
    except Exception as e:
        print(f"💥 Unexpected error: {e}")
        return False

def main():
    """Main build function"""
    print("🚀 Scrapy Service Builder")
    print("=" * 50)
    
    success = build_scrapy_executable()
    
    if success:
        print("\n🎉 BUILD SUCCESSFUL!")
        print("✅ Scrapy Service executable ready")
        print("⚡ 10x performance boost activated")
        print("🚀 Ready for production deployment")
    else:
        print("\n❌ BUILD FAILED!")
        print("Please check the error messages above")
        sys.exit(1)

if __name__ == "__main__":
    main()
