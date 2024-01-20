import "dotenv/config";
import { readFileSync, writeFileSync } from "fs";
import ky from "ky";
import { z } from "zod";

const args = process.argv[0].includes("node")
  ? process.argv.slice(2)
  : process.argv.slice(1);

async function main(): Promise<number> {
  if (args.length === 0) {
    console.log("No command selected, read the docs or use help command");
    return 1;
  }

  if (args[0].toLowerCase() === "help" || args[0].toLowerCase() === "--help") {
    console.log("TODO docs");
    return 0;
  }

  if (args[0] === "login") {
    const username = process.env.LMS_USERNAME;
    const password = process.env.LMS_PASSWORD;
    if (!username || !password) {
      console.log(
        "No username (LMS_USERNAME var) or password (LMS_PASSWORD var) found in the enviroment variables (.env file supported)"
      );
      return 1;
    }
    return await logIn(username, password);
  }

  if (args[0] === "test") {
    const refreshToken = getRefreshToken();
    if (!refreshToken) {
      console.log("Could not read refresh token from refresh-token.txt file");
      return 1;
    }
    const accessRes = await getAccessToken(refreshToken);
    if (!accessRes.success) {
      console.log("Could not get accessToken. The LMS returned an error:");
      console.log(accessRes.error);
      return 1;
    }

    writeFileSync("./refresh-token.txt", accessRes.refreshToken);

    const accessToken = accessRes.accessToken;

    const res = await ky
      .get(
        "https://api.admin.edu.goiteens.com/api/v1/training-module/additional-material/list?moduleId=17098215&groupId=17209734",
        {
          headers: {
            Authorization: `Bearer ${accessToken}`,
          },
        }
      )
      .json();
    console.log(res);
    return 0;
  }

  return 1;
}

(async () => {
  let returnCode = await main();
  process.exit(returnCode);
})();

async function logIn(username: string, password: string): Promise<number> {
  const loginResSchema = z.discriminatedUnion("success", [
    z.object({
      success: z.literal(true),
      error: z.literal("ok"),
      refreshToken: z.string(),
    }),
    z.object({
      success: z.literal(false),
      error: z.string(),
    }),
  ]);

  console.log("Logging in... It's going to take a long time");

  const rawResponse = await ky
    .post("https://api.admin.edu.goiteens.com/api/v1/auth/login", {
      json: {
        username,
        password,
        url: "https://admin.edu.goiteens.com/account/login",
      },
      timeout: 60000,
    })
    .json();

  const res = loginResSchema.parse(rawResponse);

  if (!res.success) {
    console.log("An error was returned by the LMS:");
    console.log(res.error);
    return 1;
  }

  writeFileSync("./refresh-token.txt", res.refreshToken);
  console.log(
    "Successfully logged in! You should now see an refresh-token.txt file."
  );
  console.log("It's necessary for any other command to work.");
  return 0;
}

function getRefreshToken(): string | undefined {
  try {
    return readFileSync("./refresh-token.txt", { encoding: "utf-8" });
  } catch (e) {
    return undefined;
  }
}

async function getAccessToken(refreshToken: string) {
  const refreshResSchema = z.discriminatedUnion("success", [
    z.object({
      success: z.literal(true),
      error: z.literal("ok"),
      refreshToken: z.string(),
      accessToken: z.string(),
    }),
    z.object({
      success: z.literal(false),
      error: z.string(),
    }),
  ]);

  const res = refreshResSchema.parse(
    await ky
      .post("https://api.admin.edu.goiteens.com/api/v1/auth/refresh", {
        headers: {
          Cookie: `refreshToken=${refreshToken}`,
        },
      })
      .json()
  );

  return res;
}
