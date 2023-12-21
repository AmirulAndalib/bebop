﻿using System.Runtime.Serialization;
using System;
using System.Collections.Generic;
using System.Text;
using Core.Generators.CPlusPlus;
using Core.Generators.CSharp;
using Core.Generators.Dart;
using Core.Generators.Rust;
using Core.Generators.TypeScript;
using Core.Generators.Python;
using Core.Meta;
using Core.Meta.Extensions;

namespace Core.Generators
{
    public static class GeneratorUtils
    {
        public static string GetXmlAutoGeneratedNotice()
        {
            var builder = new IndentedStringBuilder();
            builder.AppendLine("//------------------------------------------------------------------------------");
            builder.AppendLine("// <auto-generated>");
            foreach(var line in GetAutoGeneratedNotice().GetLines())
            {
                builder.Append("//").Indent(5).Append(line).Dedent(5).AppendLine();
                if (line.EndsWith("regenerated."))
                {
                    break;
                }
            }
            builder.Dedent(5).AppendLine("// </auto-generated>");
            return builder.ToString();
        }
        public static string GetMarkdownAutoGeneratedNotice()
        {
            var builder = new StringBuilder();
            builder.AppendLine("// ");
            foreach(var line in GetAutoGeneratedNotice().GetLines())
            {
                builder.Append("// ").AppendLine(line);
            }
            return builder.ToString();
        }
        private static string GetAutoGeneratedNotice()
        {
            const string repo = "https://github.com/RainwayApp/bebop";
            var builder = new IndentedStringBuilder();
            builder.AppendLine("This code was generated by a tool.").Indent(2);
            builder.AppendLine().AppendLine();
            builder.AppendLine("bebopc version:").Indent(4).AppendLine(DotEnv.Generated.Environment.Version).Dedent(4);
            builder.AppendLine().AppendLine();
            builder.AppendLine("bebopc source:").Indent(4).AppendLine(repo).Dedent(4);
            builder.AppendLine().AppendLine().Dedent(2);
            builder.AppendLine("Changes to this file may cause incorrect behavior and will be lost if").AppendLine("the code is regenerated.");
            return builder.ToString();
        }

        /// <summary>
        /// Gets the formatted base class name for a <see cref="UnionBranch"/>.
        /// </summary>
        /// <returns>The base class name.</returns>
        /// <remarks>
        ///  Used by the <see cref="CSharpGenerator"/> and other languages where "Base" indicates a class that can be inherited.
        /// </remarks>
        public static string BaseClassName(this UnionBranch branch) => $"Base{branch.Definition.Name.ToPascalCase()}";
        /// <summary>
        /// Gets the formatted class name for a <see cref="Definition"/>.
        /// </summary>
        /// <returns>The class name.</returns>
        /// <remarks>
        ///  Used by the <see cref="CSharpGenerator"/> and other languages where classes are pascal case.
        /// </remarks>
        public static string ClassName(this UnionBranch branch) => branch.Definition.ClassName();
        /// <summary>
        /// Gets the formatted class name for a <see cref="Definition"/>.
        /// </summary>
        /// <returns>The class name.</returns>
        /// <remarks>
        ///  Used by the <see cref="CSharpGenerator"/> and other languages where classes are pascal case.
        /// </remarks>
        public static string ClassName(this Definition definition)
        {
            if (definition is ServiceDefinition) {
                return $"{definition.Name.ToPascalCase()}Service";
            }
            return definition.Name.ToPascalCase();
        }

        /// <summary>
        /// Gets the formatted base class name for a <see cref="Definition"/>.
        /// </summary>
        /// <returns>The base class name.</returns>
        /// <remarks>
        ///  Used by the <see cref="CSharpGenerator"/> and other languages where "Base" indicates a class that can be inherited.
        /// </remarks>
        public static string BaseClassName(this Definition definition)
        {
            if (definition is ServiceDefinition) {
                return $"Base{definition.Name.ToPascalCase()}Service";
            }
            return $"Base{definition.Name.ToPascalCase()}";
        }

        /// <summary>
        /// Gets the generic argument index for a branch of a union.
        /// </summary>
        /// <returns>The index of the union as a generic positional argument.</returns>
        /// <remarks>
        ///  Generic arguments start at "0" whereas Bebop union branches start at "1". This just offsets the discriminator by 1 to retrieve the correct index.
        /// </remarks>
        public static int GenericIndex(this UnionBranch branch) => branch.Discriminator - 1;

        /// <summary>
        /// A dictionary that contains generators.
        /// </summary>
        /// <remarks>
        /// Generators are keyed via their commandline alias.
        /// </remarks>
        public static Dictionary<string, Func<BebopSchema, GeneratorConfig, BaseGenerator>> ImplementedGenerators  = new()
        {
            { "cpp", (s, c) => new CPlusPlusGenerator(s, c) },
            { "cs", (s, c) => new CSharpGenerator(s, c) },
            { "dart", (s, c) => new DartGenerator(s, c) },
            { "rust", (s, c) => new RustGenerator(s, c) },
            { "ts", (s, c) => new TypeScriptGenerator(s, c) },
            { "py", (s, c) => new PythonGenerator(s, c) }
        };

        public static Dictionary<string, string> ImplementedGeneratorNames  = new()
        {
            { "cpp", "C++" },
            { "cs", "C#" },
            { "dart", "Dart" },
            { "rust", "Rust" },
            { "ts", "TypeScript" },
            { "py", "Python" }
        };

        /// <summary>
        /// Returns a loop variable name based on the provided loop <paramref name="depth"/>
        /// </summary>
        /// <param name="depth">The depth of the loop</param>
        /// <returns>for 0-3 an actual letter is returned, for anything greater the depth prefixed with "i" is returned.</returns>
        public static string LoopVariable(int depth)
        {
            return $"i{depth}";
        }
    }
}
