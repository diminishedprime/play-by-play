use serde_json::{Value, json};

use super::World;
use super::ran::Ran;
use super::sut::{Running, Sut};

pub struct Claude<'a> {
    world: &'a World,
}

impl<'a> From<&'a World> for Claude<'a> {
    fn from(world: &'a World) -> Claude<'a> {
        Claude { world }
    }
}

impl Claude<'_> {
    pub async fn reads(&self, path: &str, content: &str) -> Ran {
        self.reads_from(path, content, 1).await
    }

    pub async fn reads_from(&self, path: &str, content: &str, start: usize) -> Ran {
        let lines = content.lines().count();
        let input = json!({ "file_path": path });
        let result = json!({
            "type": "text",
            "file": {
                "filePath": path,
                "content": content,
                "numLines": lines,
                "startLine": start,
                "totalLines": start - 1 + lines,
            },
        });
        self.noted("read", &input, &result).await
    }

    pub async fn writes(&self, path: &str, content: &str) -> Ran {
        let input = json!({ "file_path": path, "content": content });
        let result = json!({
            "type": "create",
            "filePath": path,
            "content": content,
            "structuredPatch": [],
            "originalFile": null,
        });
        self.noted("write", &input, &result).await
    }

    pub async fn asks_agent(&self, prompt: &str, answer: &str) -> Ran {
        let input = json!({ "description": "explore", "prompt": prompt });
        let result = json!({
            "status": "completed",
            "agentId": "a1",
            "content": [{ "text": answer }],
            "totalToolUseCount": 0,
            "totalDurationMs": 0,
            "totalTokens": 0,
            "usage": { "input_tokens": 0, "output_tokens": 0 },
            "prompt": prompt,
        });
        self.noted("agent", &input, &result).await
    }

    pub async fn edits(&self, path: &str, lines: &[&str]) -> Ran {
        let input = json!({ "file_path": path, "old_string": "", "new_string": "" });
        let removed = lines.iter().filter(|line| !line.starts_with('+')).count();
        let added = lines.iter().filter(|line| !line.starts_with('-')).count();
        let result = json!({
            "filePath": path,
            "oldString": "",
            "newString": "",
            "structuredPatch": [{
                "oldStart": 1,
                "oldLines": removed,
                "newStart": 1,
                "newLines": added,
                "lines": lines,
            }],
            "userModified": false,
            "replaceAll": false,
        });
        self.noted("edit", &input, &result).await
    }

    pub async fn searches(&self, query: &str, hits: &[(&str, &str)], summary: &str) -> Ran {
        let input = json!({ "query": query, "mode": "standard" });
        let content = hits
            .iter()
            .map(|(title, url)| json!({ "title": title, "url": url }))
            .collect::<Vec<_>>();
        let result = json!({
            "query": query,
            "results": [{ "tool_use_id": "s1", "content": content }, summary],
            "durationSeconds": 0,
        });
        self.noted("search", &input, &result).await
    }

    pub async fn fetches(&self, url: &str, prompt: &str, result: &str) -> Ran {
        let input = json!({ "url": url, "prompt": prompt });
        let output = json!({
            "bytes": result.len(),
            "code": 200,
            "codeText": "OK",
            "result": result,
            "durationMs": 0,
            "url": url,
        });
        self.noted("fetch", &input, &output).await
    }

    pub async fn runs(&self, command: &str) -> Ran {
        Sut::new(self.world.dir(), ["-c", command])
            .project(self.world.workspace())
            .within(self.world.workspace())
            .run()
            .await
    }

    pub fn starts(&self, command: &str) -> Running {
        Sut::new(self.world.dir(), ["-c", command])
            .project(self.world.workspace())
            .within(self.world.workspace())
            .start()
    }

    async fn noted(&self, kind: &str, input: &Value, result: &Value) -> Ran {
        let header = Sut::new(self.world.dir(), ["note", "header", kind])
            .project(self.world.workspace())
            .stdin(&input.to_string())
            .run()
            .await;
        let color = header.succeeds().printed().trim();
        Sut::new(self.world.dir(), ["note", "result", "--color", color, kind])
            .project(self.world.workspace())
            .stdin(&result.to_string())
            .run()
            .await
    }
}
