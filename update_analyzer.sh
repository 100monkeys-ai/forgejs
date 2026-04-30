cat << 'INNER_EOF' > /tmp/analyzer_diff.txt
<<<<<<< SEARCH
        // process.env access (shimmable → Forge.env())
        Expression::StaticMemberExpression(member) => {
            if member.property.name == "env" {
                if let Expression::Identifier(obj) = &member.object {
                    if obj.name == "process" {
                        let line = line_number_at_offset(source, member.span.start);
                        detected.push(DetectedApi {
                            pattern: "process.env".into(),
                            line,
                            compatibility: Compatibility::Shimmable,
                        });
                    }
                }
            }
            // __dirname, __filename as member access targets are covered below
        }

        // Standalone identifiers: __dirname, __filename, Buffer (as global)
=======
        // process.env access (shimmable → Forge.env())
        Expression::StaticMemberExpression(member) if member.property.name == "env" => {
            if let Expression::Identifier(obj) = &member.object {
                if obj.name == "process" {
                    let line = line_number_at_offset(source, member.span.start);
                    detected.push(DetectedApi {
                        pattern: "process.env".into(),
                        line,
                        compatibility: Compatibility::Shimmable,
                    });
                }
            }
            // __dirname, __filename as member access targets are covered below
        }
        Expression::StaticMemberExpression(_) => {}

        // Standalone identifiers: __dirname, __filename, Buffer (as global)
>>>>>>> REPLACE
INNER_EOF
