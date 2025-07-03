#!/bin/bash
# Add license headers to all source files

LICENSE_HEADER="// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2024 Lithos Systems
//
// This program is free software: you can redistribute it and/or modify
// it under the terms of the GNU Affero General Public License as published
// by the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.
//
// This program is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the
// GNU Affero General Public License for more details.
//
// You should have received a copy of the GNU Affero General Public License
// along with this program. If not, see <https://www.gnu.org/licenses/>.
"

# Find all Rust files
find src -name "*.rs" -type f | while read -r file; do
    # Check if file already has license header
    if ! grep -q "SPDX-License-Identifier" "$file"; then
        echo "Adding license header to $file"
        echo "$LICENSE_HEADER" | cat - "$file" > temp && mv temp "$file"
    fi
done

echo "✅ License headers added to all source files"
