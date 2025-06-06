#!/usr/bin/env python3
"""
🔨 Build Crawl4AI Service Executable
PyInstaller build script for creating standalone executable
"""

import os
import sys
import subprocess
import shutil
from pathlib import Path

def build_executable():
    """Build the Crawl4AI service as a standalone executable"""
    
    print("🔨 Building Crawl4AI Service Executable...")
    
    # Get current directory
    current_dir = Path(__file__).parent
    
    # Define paths
    main_script = current_dir / "main.py"
    dist_dir = current_dir / "dist"
    build_dir = current_dir / "build"
    spec_file = current_dir / "crawl4ai_service.spec"
    
    # Clean previous builds
    if dist_dir.exists():
        print("🧹 Cleaning previous dist directory...")
        shutil.rmtree(dist_dir)
    
    if build_dir.exists():
        print("🧹 Cleaning previous build directory...")
        shutil.rmtree(build_dir)
    
    if spec_file.exists():
        print("🧹 Removing previous spec file...")
        spec_file.unlink()
    
    # PyInstaller command
    pyinstaller_cmd = [
        "pyinstaller",
        "--onefile",                    # Single executable file
        "--console",                    # Console application
        "--name", "crawl4ai_service",   # Executable name
        "--clean",                      # Clean cache
        "--noconfirm",                  # Overwrite without confirmation
        
        # Hidden imports (libraries that might not be auto-detected)
        "--hidden-import", "vaderSentiment",
        "--hidden-import", "textblob",
        "--hidden-import", "bs4",
        "--hidden-import", "lxml",
        "--hidden-import", "html5lib",
        "--hidden-import", "aiohttp",
        "--hidden-import", "requests",
        
        # Data files (if needed)
        # "--add-data", "config.py:.",
        
        # Optimization
        "--optimize", "2",              # Python optimization level
        "--strip",                      # Strip debug symbols (Linux/Mac)
        
        # Exclude unnecessary modules to reduce size
        "--exclude-module", "tkinter",
        "--exclude-module", "matplotlib",
        "--exclude-module", "PIL",
        "--exclude-module", "PyQt5",
        "--exclude-module", "PyQt6",
        "--exclude-module", "PySide2",
        "--exclude-module", "PySide6",
        "--exclude-module", "jupyter",
        "--exclude-module", "notebook",
        "--exclude-module", "IPython",
        
        str(main_script)
    ]
    
    print("🚀 Running PyInstaller...")
    print(f"Command: {' '.join(pyinstaller_cmd)}")
    
    try:
        # Run PyInstaller
        result = subprocess.run(
            pyinstaller_cmd,
            cwd=current_dir,
            capture_output=True,
            text=True,
            check=True
        )
        
        print("✅ PyInstaller completed successfully!")
        
        # Check if executable was created
        executable_name = "crawl4ai_service.exe" if sys.platform == "win32" else "crawl4ai_service"
        executable_path = dist_dir / executable_name
        
        if executable_path.exists():
            file_size = executable_path.stat().st_size / (1024 * 1024)  # MB
            print(f"✅ Executable created: {executable_path}")
            print(f"📦 File size: {file_size:.1f} MB")
            
            # Test the executable
            print("🧪 Testing executable...")
            test_result = test_executable(executable_path)
            
            if test_result:
                print("✅ Executable test passed!")
                return True
            else:
                print("❌ Executable test failed!")
                return False
        else:
            print("❌ Executable not found after build!")
            return False
            
    except subprocess.CalledProcessError as e:
        print(f"❌ PyInstaller failed with error code {e.returncode}")
        print(f"STDOUT: {e.stdout}")
        print(f"STDERR: {e.stderr}")
        return False
    except Exception as e:
        print(f"❌ Build failed with error: {e}")
        return False


def test_executable(executable_path: Path) -> bool:
    """Test the built executable with a simple request"""
    
    import json
    import tempfile
    
    # Create test input
    test_input = {
        "token_symbol": "SOL",
        "data_types": ["news"],
        "time_range_hours": 1,
        "max_results": 5,
        "sentiment_analysis": True
    }
    
    try:
        # Write test input to temporary file
        with tempfile.NamedTemporaryFile(mode='w', suffix='.json', delete=False) as f:
            json.dump(test_input, f)
            input_file = f.name
        
        # Run executable with test input
        result = subprocess.run(
            [str(executable_path)],
            input=json.dumps(test_input),
            capture_output=True,
            text=True,
            timeout=30  # 30 second timeout
        )
        
        # Clean up temp file
        os.unlink(input_file)
        
        # Check result
        if result.returncode == 0:
            try:
                output = json.loads(result.stdout)
                if output.get('status') in ['success', 'error']:
                    return True
            except json.JSONDecodeError:
                pass
        
        print(f"Test output: {result.stdout}")
        print(f"Test errors: {result.stderr}")
        return False
        
    except subprocess.TimeoutExpired:
        print("❌ Executable test timed out")
        return False
    except Exception as e:
        print(f"❌ Executable test failed: {e}")
        return False


def install_dependencies():
    """Install required dependencies"""
    
    print("📦 Installing dependencies...")
    
    requirements_file = Path(__file__).parent / "requirements.txt"
    
    if not requirements_file.exists():
        print("❌ requirements.txt not found!")
        return False
    
    try:
        subprocess.run([
            sys.executable, "-m", "pip", "install", "-r", str(requirements_file)
        ], check=True)
        
        print("✅ Dependencies installed successfully!")
        return True
        
    except subprocess.CalledProcessError as e:
        print(f"❌ Failed to install dependencies: {e}")
        return False


def main():
    """Main build process"""
    
    print("🚀 Crawl4AI Service Build Process")
    print("=" * 50)
    
    # Check if we're in the right directory
    current_dir = Path(__file__).parent
    main_script = current_dir / "main.py"
    
    if not main_script.exists():
        print("❌ main.py not found! Make sure you're in the correct directory.")
        sys.exit(1)
    
    # Install dependencies
    if not install_dependencies():
        print("❌ Failed to install dependencies!")
        sys.exit(1)
    
    # Build executable
    if build_executable():
        print("\n🎉 Build completed successfully!")
        print(f"📁 Executable location: {current_dir / 'dist'}")
        print("\n📋 Next steps:")
        print("1. Test the executable with real data")
        print("2. Integrate with Rust SniperBot")
        print("3. Deploy to production environment")
    else:
        print("\n❌ Build failed!")
        sys.exit(1)


if __name__ == "__main__":
    main()
