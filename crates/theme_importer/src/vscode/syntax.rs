use indexmap::IndexMap;
use serde::Deserialize;
use strum::EnumIter;

#[derive(Debug, PartialEq, Eq, Deserialize)]
#[serde(untagged)]
pub enum VsCodeTokenScope {
    One(String),
    Many(Vec<String>),
}

#[derive(Debug, Deserialize)]
pub struct VsCodeTokenColor {
    pub name: Option<String>,
    pub scope: Option<VsCodeTokenScope>,
    pub settings: VsCodeTokenColorSettings,
}

#[derive(Debug, Deserialize)]
pub struct VsCodeTokenColorSettings {
    pub foreground: Option<String>,
    pub background: Option<String>,
    #[serde(rename = "fontStyle")]
    pub font_style: Option<String>,
}

#[derive(Debug, PartialEq, Copy, Clone, EnumIter)]
pub enum EditsyncSyntaxToken {
    Attribute,
    Boolean,
    Comment,
    CommentDoc,
    Constant,
    Constructor,
    Embedded,
    Emphasis,
    EmphasisStrong,
    Enum,
    Function,
    Hint,
    Keyword,
    Label,
    LinkText,
    LinkUri,
    Number,
    Operator,
    Predictive,
    Preproc,
    Primary,
    Property,
    Punctuation,
    PunctuationBracket,
    PunctuationDelimiter,
    PunctuationListMarker,
    PunctuationSpecial,
    String,
    StringEscape,
    StringRegex,
    StringSpecial,
    StringSpecialSymbol,
    Tag,
    TextLiteral,
    Title,
    Type,
    Variable,
    VariableSpecial,
    Variant,
}

impl std::fmt::Display for EditsyncSyntaxToken {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match self {
                EditsyncSyntaxToken::Attribute => "attribute",
                EditsyncSyntaxToken::Boolean => "boolean",
                EditsyncSyntaxToken::Comment => "comment",
                EditsyncSyntaxToken::CommentDoc => "comment.doc",
                EditsyncSyntaxToken::Constant => "constant",
                EditsyncSyntaxToken::Constructor => "constructor",
                EditsyncSyntaxToken::Embedded => "embedded",
                EditsyncSyntaxToken::Emphasis => "emphasis",
                EditsyncSyntaxToken::EmphasisStrong => "emphasis.strong",
                EditsyncSyntaxToken::Enum => "enum",
                EditsyncSyntaxToken::Function => "function",
                EditsyncSyntaxToken::Hint => "hint",
                EditsyncSyntaxToken::Keyword => "keyword",
                EditsyncSyntaxToken::Label => "label",
                EditsyncSyntaxToken::LinkText => "link_text",
                EditsyncSyntaxToken::LinkUri => "link_uri",
                EditsyncSyntaxToken::Number => "number",
                EditsyncSyntaxToken::Operator => "operator",
                EditsyncSyntaxToken::Predictive => "predictive",
                EditsyncSyntaxToken::Preproc => "preproc",
                EditsyncSyntaxToken::Primary => "primary",
                EditsyncSyntaxToken::Property => "property",
                EditsyncSyntaxToken::Punctuation => "punctuation",
                EditsyncSyntaxToken::PunctuationBracket => "punctuation.bracket",
                EditsyncSyntaxToken::PunctuationDelimiter => "punctuation.delimiter",
                EditsyncSyntaxToken::PunctuationListMarker => "punctuation.list_marker",
                EditsyncSyntaxToken::PunctuationSpecial => "punctuation.special",
                EditsyncSyntaxToken::String => "string",
                EditsyncSyntaxToken::StringEscape => "string.escape",
                EditsyncSyntaxToken::StringRegex => "string.regex",
                EditsyncSyntaxToken::StringSpecial => "string.special",
                EditsyncSyntaxToken::StringSpecialSymbol => "string.special.symbol",
                EditsyncSyntaxToken::Tag => "tag",
                EditsyncSyntaxToken::TextLiteral => "text.literal",
                EditsyncSyntaxToken::Title => "title",
                EditsyncSyntaxToken::Type => "type",
                EditsyncSyntaxToken::Variable => "variable",
                EditsyncSyntaxToken::VariableSpecial => "variable.special",
                EditsyncSyntaxToken::Variant => "variant",
            }
        )
    }
}

impl EditsyncSyntaxToken {
    pub fn find_best_token_color_match<'a>(
        &self,
        token_colors: &'a [VsCodeTokenColor],
    ) -> Option<&'a VsCodeTokenColor> {
        let mut ranked_matches = IndexMap::new();

        for (ix, token_color) in token_colors.iter().enumerate() {
            if token_color.settings.foreground.is_none() {
                continue;
            }

            let Some(rank) = self.rank_match(token_color) else {
                continue;
            };

            if rank > 0 {
                ranked_matches.insert(ix, rank);
            }
        }

        ranked_matches
            .into_iter()
            .max_by_key(|(_, rank)| *rank)
            .map(|(ix, _)| &token_colors[ix])
    }

    fn rank_match(&self, token_color: &VsCodeTokenColor) -> Option<u32> {
        let candidate_scopes = match token_color.scope.as_ref()? {
            VsCodeTokenScope::One(scope) => vec![scope],
            VsCodeTokenScope::Many(scopes) => scopes.iter().collect(),
        }
        .iter()
        .map(|scope| scope.as_str())
        .collect::<Vec<_>>();

        let scopes_to_match = self.to_vscode();
        let number_of_scopes_to_match = scopes_to_match.len();

        let mut matches = 0;

        for (ix, scope) in scopes_to_match.into_iter().enumerate() {
            // Assign each entry a weight that is inversely proportional to its
            // position in the list.
            //
            // Entries towards the front are weighted higher than those towards the end.
            let weight = (number_of_scopes_to_match - ix) as u32;

            if candidate_scopes.contains(&scope) {
                matches += 1 + weight;
            }
        }

        Some(matches)
    }

    pub fn fallbacks(&self) -> &[Self] {
        match self {
            EditsyncSyntaxToken::CommentDoc => &[EditsyncSyntaxToken::Comment],
            EditsyncSyntaxToken::Number => &[EditsyncSyntaxToken::Constant],
            EditsyncSyntaxToken::VariableSpecial => &[EditsyncSyntaxToken::Variable],
            EditsyncSyntaxToken::PunctuationBracket
            | EditsyncSyntaxToken::PunctuationDelimiter
            | EditsyncSyntaxToken::PunctuationListMarker
            | EditsyncSyntaxToken::PunctuationSpecial => &[EditsyncSyntaxToken::Punctuation],
            EditsyncSyntaxToken::StringEscape
            | EditsyncSyntaxToken::StringRegex
            | EditsyncSyntaxToken::StringSpecial
            | EditsyncSyntaxToken::StringSpecialSymbol => &[EditsyncSyntaxToken::String],
            _ => &[],
        }
    }

    fn to_vscode(self) -> Vec<&'static str> {
        match self {
            EditsyncSyntaxToken::Attribute => vec!["entity.other.attribute-name"],
            EditsyncSyntaxToken::Boolean => vec!["constant.language"],
            EditsyncSyntaxToken::Comment => vec!["comment"],
            EditsyncSyntaxToken::CommentDoc => vec!["comment.block.documentation"],
            EditsyncSyntaxToken::Constant => vec!["constant", "constant.language", "constant.character"],
            EditsyncSyntaxToken::Constructor => {
                vec![
                    "entity.name.tag",
                    "entity.name.function.definition.special.constructor",
                ]
            }
            EditsyncSyntaxToken::Embedded => vec!["meta.embedded"],
            EditsyncSyntaxToken::Emphasis => vec!["markup.italic"],
            EditsyncSyntaxToken::EmphasisStrong => vec![
                "markup.bold",
                "markup.italic markup.bold",
                "markup.bold markup.italic",
            ],
            EditsyncSyntaxToken::Enum => vec!["support.type.enum"],
            EditsyncSyntaxToken::Function => vec![
                "entity.function",
                "entity.name.function",
                "variable.function",
            ],
            EditsyncSyntaxToken::Hint => vec![],
            EditsyncSyntaxToken::Keyword => vec![
                "keyword",
                "keyword.other.fn.rust",
                "keyword.control",
                "keyword.control.fun",
                "keyword.control.class",
                "punctuation.accessor",
                "entity.name.tag",
            ],
            EditsyncSyntaxToken::Label => vec![
                "label",
                "entity.name",
                "entity.name.import",
                "entity.name.package",
            ],
            EditsyncSyntaxToken::LinkText => vec!["markup.underline.link", "string.other.link"],
            EditsyncSyntaxToken::LinkUri => vec!["markup.underline.link", "string.other.link"],
            EditsyncSyntaxToken::Number => vec!["constant.numeric", "number"],
            EditsyncSyntaxToken::Operator => vec!["operator", "keyword.operator"],
            EditsyncSyntaxToken::Predictive => vec![],
            EditsyncSyntaxToken::Preproc => vec![
                "preproc",
                "meta.preprocessor",
                "punctuation.definition.preprocessor",
            ],
            EditsyncSyntaxToken::Primary => vec![],
            EditsyncSyntaxToken::Property => vec![
                "variable.member",
                "support.type.property-name",
                "variable.object.property",
                "variable.other.field",
            ],
            EditsyncSyntaxToken::Punctuation => vec![
                "punctuation",
                "punctuation.section",
                "punctuation.accessor",
                "punctuation.separator",
                "punctuation.definition.tag",
            ],
            EditsyncSyntaxToken::PunctuationBracket => vec![
                "punctuation.bracket",
                "punctuation.definition.tag.begin",
                "punctuation.definition.tag.end",
            ],
            EditsyncSyntaxToken::PunctuationDelimiter => vec![
                "punctuation.delimiter",
                "punctuation.separator",
                "punctuation.terminator",
            ],
            EditsyncSyntaxToken::PunctuationListMarker => {
                vec!["markup.list punctuation.definition.list.begin"]
            }
            EditsyncSyntaxToken::PunctuationSpecial => vec!["punctuation.special"],
            EditsyncSyntaxToken::String => vec!["string"],
            EditsyncSyntaxToken::StringEscape => {
                vec!["string.escape", "constant.character", "constant.other"]
            }
            EditsyncSyntaxToken::StringRegex => vec!["string.regex"],
            EditsyncSyntaxToken::StringSpecial => vec!["string.special", "constant.other.symbol"],
            EditsyncSyntaxToken::StringSpecialSymbol => {
                vec!["string.special.symbol", "constant.other.symbol"]
            }
            EditsyncSyntaxToken::Tag => vec!["tag", "entity.name.tag", "meta.tag.sgml"],
            EditsyncSyntaxToken::TextLiteral => vec!["text.literal", "string"],
            EditsyncSyntaxToken::Title => vec!["title", "entity.name"],
            EditsyncSyntaxToken::Type => vec![
                "entity.name.type",
                "entity.name.type.primitive",
                "entity.name.type.numeric",
                "keyword.type",
                "support.type",
                "support.type.primitive",
                "support.class",
            ],
            EditsyncSyntaxToken::Variable => vec![
                "variable",
                "variable.language",
                "variable.member",
                "variable.parameter",
                "variable.parameter.function-call",
            ],
            EditsyncSyntaxToken::VariableSpecial => vec![
                "variable.special",
                "variable.member",
                "variable.annotation",
                "variable.language",
            ],
            EditsyncSyntaxToken::Variant => vec!["variant"],
        }
    }
}
