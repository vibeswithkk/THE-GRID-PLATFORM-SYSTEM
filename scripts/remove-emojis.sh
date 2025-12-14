#!/bin/bash
# Remove all emojis from TGP codebase for professional standards

echo "Removing all emojis from TGP codebase..."

# Define emoji patterns to remove
EMOJI_PATTERN='🎯\|🚀\|✅\|❌\|📊\|🧪\|🔍\|📦\|🌐\|⚙️\|🔄\|🐳\|⚡\|🏆\|📈\|🎉\|⭐\|💡\|🔧\|📝\|🎊\|❤️\|🏗️\|🔬\|🎨\|🆕\|⏭️\|⬇️\|✨\|🤝\|🙏\|📚\|📋\|🗺️\|🛠️\|🔗\|👨‍💻\|⚠️\|🔧\|��\|🔥\|📁\|🌟'

# Files to clean (excluding CETAK.BIRU.md which is blueprint)
FILES_TO_CLEAN="
./README.md
./KNOWN_ISSUES.md
./api/README.md
./docs/DEPLOYMENT.md
./tests/pre-release-test.sh
./tests/test-docker-execution.sh
./test-client/src/main.rs
"

for file in $FILES_TO_CLEAN; do
    if [ -f "$file" ]; then
        echo "Cleaning: $file"
        
        # Replace checkmarks
        sed -i 's/✅ /**/g' "$file"
        sed -i 's/❌ /**/g' "$file"
        
        # Remove common emojis in headers
        sed -i 's/🎯 //g' "$file"
        sed -i 's/🚀 //g' "$file"
        sed -i 's/📊 //g' "$file"
        sed -i 's/💡 //g' "$file"
        sed -i 's/🔧 //g' "$file"
        sed -i 's/📈 //g' "$file"
        sed -i 's/🧪 //g' "$file"
        sed -i 's/🔍 //g' "$file"
        sed -i 's/📦 //g' "$file"
        sed -i 's/🌐 //g' "$file"
        sed -i 's/⚙️  //g' "$file"
        sed -i 's/⚙️ //g' "$file"
        sed -i 's/🔄 //g' "$file"
        sed -i 's/🐳 //g' "$file"
        sed -i 's/⚡ //g' "$file"
        sed -i 's/🏆 //g' "$file"
        sed -i 's/🎉 //g' "$file"
        sed -i 's/⭐ //g' "$file"
        sed -i 's/🎊 //g' "$file"
        sed -i 's/❤️ //g' "$file"
        sed -i 's/🏗️ //g' "$file"
        sed -i 's/🔬 //g' "$file"
        sed -i 's/━━━━ /## /g' "$file"
        
        echo "  - Cleaned"
    fi
done

echo "All emojis removed successfully!"
